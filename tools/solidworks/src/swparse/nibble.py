"""Nibble-swap encoding used for stream names in SolidWorks 3DEXPERIENCE files.

Each byte has its high and low 4-bit nibbles swapped:
    0xAB -> 0xBA

Example:
    Raw bytes:  34 f6 e6 47
    Swapped:    43 6f 6e 74 = "Cont"
"""


def swap(data: bytes) -> bytes:
    """Swap nibbles in every byte.  Applying twice is identity."""
    return bytes([((b >> 4) | (b << 4)) & 0xFF for b in data])


def decode_name(raw: bytes) -> str:
    """Nibble-swap *raw* bytes then decode as ASCII."""
    return swap(raw).decode("ascii", errors="replace")


def encode_name(name: str) -> bytes:
    """Encode an ASCII name into nibble-swapped bytes."""
    return swap(name.encode("ascii"))
