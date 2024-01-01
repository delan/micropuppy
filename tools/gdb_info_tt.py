from __future__ import annotations
from dataclasses import dataclass

# pyright: reportMissingModuleSource=false
import gdb


class InfoTtCommand(gdb.Command):
    """Print translation table.
    info tt [address level]

    Prints the translation table pointed to by $TTBR0_EL1 by default (level 0).

    Duplicate tables are never printed twice: the second occurrence of any table will be replaced
    with an ellipsis.

    Legend:
        📖: table descriptor
        🧱: block descriptor
        🧾: page descriptor
    """

    def __init__(self):
        super().__init__("info tt", gdb.COMMAND_STATUS)

    def invoke(self, argument, from_tty):
        inferior = gdb.inferiors()[0]
        argument = gdb.string_to_argv(argument)

        if len(argument) == 0:
            address = "$TTBR0_EL1"
            level = 0
        elif len(argument) == 1:
            raise RuntimeError("missing level argument")
        elif len(argument) == 2:
            address = argument[0]
            level = gdb.parse_and_eval(argument[1])
        else:
            raise RuntimeError("too many arguments")

        info_tt(inferior, address, level)


class InfoTtAlias(gdb.Command):
    def __init__(self, name, address):
        super().__init__(f"info {name}", gdb.COMMAND_STATUS)
        self.address = address

    def invoke(self, argument, from_tty):
        inferior = gdb.inferiors()[0]
        argument = gdb.string_to_argv(argument)

        address = self.address
        if len(argument) == 0:
            level = 0
        elif len(argument) == 1:
            level = gdb.parse_and_eval(argument[0])
        else:
            raise RuntimeError("too many arguments")

        info_tt(inferior, address, level)


class InfoTt0(InfoTtAlias):
    """Print translation table for $TTBR0_EL1.
    info tt0 [level]

    Same as “info tt $TTBR0_EL1”.
    """

    def __init__(self):
        super().__init__("tt0", "$TTBR0_EL1")


class InfoTt1(InfoTtAlias):
    """Print translation table for $TTBR1_EL1.
    info tt1 [level]

    Same as “info tt $TTBR1_EL1”.
    """

    def __init__(self):
        super().__init__("tt1", "$TTBR1_EL1")


def info_tt(inferior, address, level):
        # TODO be smarter than this, or maybe even be dumber (always 0xFFFF...)?
        starting_va = 0xFFFF_FFFF_FFFF_FFFF if address == "$TTBR1_EL1" else 0
        address = gdb.parse_and_eval(address)
        print(table_str_from_inferior(inferior, int(address), int(level), starting_va))


def table_str_from_inferior(inferior, address, level, starting_va):
    VA_BITS = 48
    LEVELS = 4
    PAGE_SIZE_EXP = 12
    EACH_LEVEL_BITS = PAGE_SIZE_EXP - Descriptor.SIZE_EXP
    TABLE_LEN = 2 ** EACH_LEVEL_BITS
    TABLE_SIZE = TABLE_LEN * Descriptor.SIZE
    assert TABLE_SIZE == 2 ** PAGE_SIZE_EXP

    # current effective values
    assert EACH_LEVEL_BITS == 9
    assert TABLE_LEN == 512

    visited = set()

    def table_str_from_inferior(inferior, address, level, starting_va):
        heading = f"{TableDescriptor(address)} [level {level}]"
        index_width = 0
        starting_vas = [starting_va]
        entries = []

        if address not in visited:
            visited.add(address)

            try:
                descriptors = inferior.read_memory(address, TABLE_SIZE).tobytes()
            except Exception as e:
                raise Exception(f"could not read table at {pretty_hex(address)}: {e}")

            descriptors = [
                Descriptor.from_bytes(
                    descriptors[i * Descriptor.SIZE : (i + 1) * Descriptor.SIZE]
                ).parse(level)
                for i in range(TABLE_LEN)
            ]

            max_index = len(descriptors) - 1
            index_width = len(str(max_index))
            i = 0
            while i < len(descriptors):
                descriptor = descriptors[i]
                entry = f"[{i:<{index_width}}]"

                # FIXME unsure if min(VA_BITS) is the right way to deal with smaller T0SZ/T1SZ,
                # so for now let’s assert that we haven’t changed VA_BITS
                clear_shift = PAGE_SIZE_EXP + EACH_LEVEL_BITS * (LEVELS - level)
                index_shift = clear_shift - EACH_LEVEL_BITS
                assert clear_shift <= VA_BITS
                starting_va &= ~((1 << min(VA_BITS, clear_shift)) - 1)
                starting_va |= i << min(VA_BITS, index_shift)
                starting_vas.append(starting_va)

                i += 1
                if isinstance(descriptor, TableDescriptor):
                    entry_starting_vas, content = table_str_from_inferior(
                        inferior, descriptor.table_address, level + 1, starting_va
                    )
                    starting_vas.pop()
                    starting_vas.extend(entry_starting_vas)
                    entry += f": {content}"
                elif descriptor is not None:
                    # BlockDescriptor or PageDescriptor
                    entry += f": {descriptor}"
                else:
                    next_index = i
                    while i < len(descriptors) and descriptors[i] is None:
                        i += 1
                    if i > next_index:
                        entry += f"-[{(i-1):<{index_width}}]"
                    entry += ": 😶 <none>"

                entries.append(entry)
        else:
            heading += " (dup)"

        return (starting_vas, pretty_tree(heading, entries, index_width))

    starting_vas, result = table_str_from_inferior(inferior, address, level, starting_va)
    lines = result.splitlines()
    assert len(starting_vas) == len(lines), f"starting_vas != lines ({len(starting_vas)} != {len(lines)})"
    return "\n".join(f"{pretty_hex(va)} {line}" for va, line in zip(starting_vas, lines))


@dataclass
class Descriptor:
    SIZE_EXP = 3
    SIZE = 2 ** SIZE_EXP
    descriptor: bytes

    @staticmethod
    def from_bytes(descriptor_bytes):
        assert len(descriptor_bytes) == Descriptor.SIZE

        return Descriptor(int.from_bytes(descriptor_bytes, byteorder="little"))

    def parse(self, level):
        if not self.bit(0):  # 'valid' bit
            return None
        elif not self.bit(1):
            return BlockDescriptor.from_descriptor(self, level)
        elif level == 3:
            return PageDescriptor.from_descriptor(self)
        else:
            return TableDescriptor.from_descriptor(self)

    def bit(self, index):
        return (self.descriptor >> index) & 1

    def field_unshifted(self, high, low):
        mask = (1 << (high - low + 1)) - 1
        return self.descriptor & (mask << low)


@dataclass
class BlockDescriptor:
    output_address: int
    attr_AF: bool

    @staticmethod
    def from_descriptor(descriptor, level):
        if level == 1:
            n = 30
        elif level == 2:
            n = 21
        else:
            return "[invalid level]"

        output_address = descriptor.field_unshifted(47, n)
        attr_AF = int(descriptor.bit(10))

        return BlockDescriptor(output_address, attr_AF)

    def __str__(self):
        return f"🧱 {pretty_hex(self.output_address)} AF={self.attr_AF}"


@dataclass
class PageDescriptor:
    output_address: int

    @staticmethod
    def from_descriptor(descriptor, level):
        output_address = descriptor.field_unshifted(47, 12)

        return PageDescriptor(output_address)

    def __str__(self):
        return f"🧾 {pretty_hex(self.output_address)}"


@dataclass
class TableDescriptor:
    table_address: int

    @staticmethod
    def from_descriptor(descriptor):
        m = 12

        table_address = descriptor.field_unshifted(47, m)

        return TableDescriptor(table_address)

    def __str__(self):
        return f"📖 {pretty_hex(self.table_address)}"


def pretty_hex(value, *, width=64):
    width_nibbles = (width + 3) // 4
    value = f"{value:0{width_nibbles}x}"

    offcut_len = len(value) % 4
    offcut, nibbles = value[:offcut_len], value[offcut_len:]

    nibbles = "_".join(nibbles[i * 4 : (i + 1) * 4] for i in range(len(nibbles) // 4))
    if len(offcut) == 0:
        return f"0x{nibbles}"
    else:
        return f"0x{offcut}_{nibbles}"


def pretty_tree(heading, entries, index_width):
    BOX_NS = "\u2502"
    BOX_NE = "\u2514"
    BOX_NSE = "\u251c"

    max_index = len(entries) - 1

    def format_entry(index, entry):
        prefixes = []
        lines = entry.splitlines()

        # prefix for first line
        if len(lines) == 1 and index == max_index:
            # this is the last box drawing character of the tree
            box_first = BOX_NE
        else:
            box_first = BOX_NSE
        prefix_first = f"{box_first} "  # index provided by caller
        prefixes.append(prefix_first)

        # prefix for lines between the first and last line
        if len(lines) > 2:
            box_mid = BOX_NS
            prefix_mid = f"{box_mid} {' ' * (index_width + 3)}"
            prefixes.extend([prefix_mid] * (len(lines) - 2))

        # prefix for last line (only if the last line isn't the first line)
        if len(lines) > 1:
            if index == max_index:
                # this is the last box drawing character of the tree
                box_last = BOX_NE
            else:
                box_last = BOX_NS
            prefix_last = f"{box_last} {' ' * (index_width + 3)}"
            prefixes.append(prefix_last)

        return "\n".join(f"{prefix} {line}" for prefix, line in zip(prefixes, lines))

    formatted_entries = [
        format_entry(index, entry) for index, entry in enumerate(entries)
    ]
    return "\n".join([heading] + formatted_entries)


InfoTtCommand()
InfoTt0()
InfoTt1()
