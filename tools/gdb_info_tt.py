BOX_N = "\u2575"
BOX_NS = "\u2502"
BOX_NE = "\u2514"
BOX_NSE = "\u251c"


class InfoTtCommand(gdb.Command):
    """Print translation table.
    info tt [address [level]]

    Prints $TTBR0_EL1 by default.
    """

    def __init__(self):
        super().__init__("info tt", gdb.COMMAND_STATUS)

    def invoke(self, argument, from_tty):
        inferior = gdb.inferiors()[0]
        argument = gdb.string_to_argv(argument)

        if len(argument) == 0:
            base_address = gdb.parse_and_eval("$TTBR0_EL1")
            level = 0
        elif len(argument) == 1:
            base_address = gdb.parse_and_eval(argument[0])
            level = 0
        elif len(argument) == 2:
            base_address = gdb.parse_and_eval(argument[0])
            level = gdb.parse_and_eval(argument[1])
        else:
            raise "too many arguments"

        table = Table(inferior, int(base_address), level=int(level))
        print(table.as_str(set()))


class Table:
    SIZE = 4096

    def __init__(self, inferior, base_address, *, level):
        self.inferior = inferior
        self.base_address = base_address
        self.table = inferior.read_memory(base_address, self.SIZE)
        self.level = level

    def as_str(self, visited):
        def descriptor_as_str(index):
            start = index * Descriptor.SIZE
            end = start + Descriptor.SIZE
            descriptor = Descriptor(
                self.inferior, self.table[start:end], level=self.level + 1
            )

            s = descriptor.as_str(visited)
            lines = s.splitlines()

            # generate line box drawing character prefixes
            prefixes = []

            box_first = BOX_NSE
            prefix_first = f"{box_first} [{index:<3}]:"
            prefixes.append(prefix_first)

            if len(lines) > 2:
                box_mid = BOX_NS
                prefix_mid = f"{box_mid}       "
                prefixes.extend([prefix_mid] * (len(lines) - 2))

            if len(lines) > 1:
                if index != 511:
                    box_last = BOX_NS
                else:
                    box_last = BOX_N
                prefix_last = f"{box_last}       "
                prefixes.append(prefix_last)

            return "\n".join(
                f"   {prefix} {line}" for prefix, line in zip(prefixes, lines)
            )

        header = f"🧾 ({self.base_address:#018x}) level {self.level}"
        if self.base_address in visited:
            return f"{header} ..."
        else:
            visited.add(self.base_address)

            entries = [descriptor_as_str(index) for index in range(512)]
            return "\n".join([header, *entries])


class Descriptor:
    SIZE = 8

    def __init__(self, inferior, descriptor, *, level):
        assert len(descriptor) == self.SIZE

        self.inferior = inferior
        self.descriptor = int.from_bytes(descriptor, byteorder="little")
        self.level = level

    def as_str(self, visited):
        if self.bit(0) == 0:
            return

        if self.bit(1) == 0:
            # D_Block
            if self.level == 1:
                n = 30
            elif self.level == 2:
                n = 21
            else:
                return "frick"

            output_address = self.field_unshifted(47, n)
            AF = self.bit(10)

            return f"🧱 {output_address:#018x} {AF=}"
        else:
            # D_Table or D_Page
            m = 12
            base_address = self.field_unshifted(47, m)

            table = Table(self.inferior, base_address, level=self.level)
            return table.as_str(visited)

    def bit(self, index):
        return (self.descriptor >> index) & 1

    def field_unshifted(self, high, low):
        mask = (1 << (high - low + 1)) - 1
        return self.descriptor & (mask << low)


InfoTtCommand()
