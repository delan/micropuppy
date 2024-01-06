import atexit
from dataclasses import dataclass
import subprocess

# pyright: reportMissingModuleSource=false
import gdb


@dataclass
class Qemu:
    process: subprocess.Popen | None = None

    def is_running(self):
        return (self.process is not None) and (self.process.returncode is None)

    def run(self, interactive=True):
        if not self.is_running():
            # stdin=subprocess.DEVNULL prevents GDB from becoming massively slow (???)
            #
            # process_group=0 prevents qemu from receiving ^C (intended to break into the target)
            # and exiting
            self.process = subprocess.Popen(
                ["./run", "qemu", "-d"], stdin=subprocess.DEVNULL, process_group=0
            )
            print("qemu started")
        elif interactive:
            raise gdb.GdbError("error: qemu already running")

    def terminate(self, interactive=True):
        if self.is_running():
            self.process.terminate()
            self.process = None
            print("qemu terminated")
        elif interactive:
            raise gdb.GdbError("error: qemu not running")

    def attach(self):
        gdb.execute("target extended-remote :1234")

    def detach(self):
        gdb.execute("disconnect")


qemu = Qemu()
atexit.register(lambda: qemu.terminate(interactive=False))


class QemuCommand(gdb.Command):
    """Prefix command for invoking QEMU."""

    def __init__(self):
        super().__init__("qemu", gdb.COMMAND_NONE, gdb.COMPLETE_NONE, prefix=True)

    def invoke(self, argument, from_tty):
        gdb.execute("help qemu")


class QemuRunCommand(gdb.Command):
    def __init__(self):
        # COMMAND_RUNNING too, if someday gdb supports multiple classes
        super().__init__("qemu run", gdb.COMMAND_USER)

    def invoke(self, argument, from_tty):
        qemu.run()
        qemu.attach()
        self.dont_repeat()


class QemuTerminateCommand(gdb.Command):
    def __init__(self):
        # COMMAND_RUNNING too, if someday gdb supports multiple classes
        super().__init__("qemu terminate", gdb.COMMAND_USER)

    def invoke(self, argument, from_tty):
        qemu.detach()
        qemu.terminate()
        self.dont_repeat()


QemuCommand()
QemuRunCommand()
QemuTerminateCommand()
