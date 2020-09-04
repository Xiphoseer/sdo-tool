# Damaged files

The Signum! 1/2 file format relies a lot on run sequences of run-length encoded
data. That makes it somewhat brittle, because if a single on of those length
specifiers is off, there is no reliable way to know where the next valid part is.

