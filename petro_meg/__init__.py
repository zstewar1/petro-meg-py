"""Provides reading and writing support for Petroglyph's MEGA file format."""

from .petro_meg import MegPath, FileEntry, read_meg
from .writer import MegBuilder
