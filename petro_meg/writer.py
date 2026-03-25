import io

from .petro_meg import MegPath, _MegBuilder


class MegBuilder(object):
    """Builder for MEGA files."""

    def __init__(self, version, /, key=None, iv=None, entries=None):
        """Create a MEGA file builder with the given version."""
        self._entries = {}
        self.version = version
        self.set_encryption(key, iv)
        if entries is not None:
            self.update(entries)

    def set_encryption(self, key, iv=None):
        """Sets the Key and Initial Vector used for encryption or clears them.

        For convenience this can be called with a single argument, `set_encryption(None)` to clear
        encryption. To enable encryption both Key and IV must be specified.
        """
        match key, iv:
            case (None, None) | ((None, None), None):
                self._key = None
                self._iv = None
            case ((key, iv), None) | (key, iv):
                key = bytes(key)
                iv = bytes(iv)
                if len(key) != 16 or len(iv) != 16:
                    raise ValueError('Key and IV must both have len 16')
                self._key = key
                self._iv = iv
            case _:
                raise TypeError('Key and IV must eith both be specified or both be None, or key must be a tuple containing both the Key and IV')

    def get_encryption(self):
        """Returns a tuple containing the current Key and IV used for encryption."""
        return self._key, self._iv

    @property
    def key(self):
        """Gets the Key used for encryption."""
        return self._key

    @property
    def iv(self):
        """Gets the Initial Vector used for encryption."""
        return self._iv

    @property
    def version(self):
        return self._version

    @version.setter
    def version(self, version):
        match version:
            case 1 | '1' | 'v1' | 'V1':
                self._version = 'V1'
            case 2 | '2' | 'v2' | 'V2':
                self._version = 'V2'
            case 3 | '3' | 'v3' | 'V3':
                self._version = 'V3'
            case _:
                if not (isinstance(version, str) or isinstance(version, int)):
                    raise TypeError(f'Version must be int or str, got {type(version)}')
                raise ValueError(f'Version must be 1, 2, or 3, got {version}')

    def get(self, path, default=None):
        """Gets the item with the given path or fallback to a default."""
        path = MegPath(path)
        return self._entries.get(path, default)

    def __getitem__(self, path):
        """Gets the entry with the given path or errors if there isn't one."""
        path = MegPath(path)
        return self._entries[path]

    def __setitem__(self, path, value):
        """Sets the entry with the given path."""
        path = MegPath(path)
        self._entries[path] = value

    def update(self, values=None, **kwargs):
        """Updates several entries in the builder at once."""
        if values is not None:
            # Convert to dict once to handle e.g. lists of tuples.
            values = dict(values)
            # Then convert the keys to MegPaths.
            self._entries.update({MegPath(k): v for k, v in values.items()})
        self._entries.update({MegPath(k): v for k, v in values.items()})


    def __delitem__(self, path):
        """Deletes the entry with the given path."""
        path = MegPath(path)
        del self._entries[path]

    def __contains__(self, path):
        """Checks if this builder contains an etry with the given path."""
        path = MegPath(path)
        return path in self._entries

    def __iter__(self):
        """Gets an iterator over the files contained in this Builder."""
        return iter(self._entries)

    def keys(self):
        """Gets an iterator over the keys in the Builder."""
        return self._entries.keys()

    def values(self):
        """Gets an iterator over the values in the Builder."""
        return self._entries.values()

    def items(self):
        """Gets an iterator over the items in the Builder."""
        return self._entries.items()

    def build(self, outfile=None):
        """Builds the MEGA file.

        Writes the output to the given file, or returns a bytes object with the contents if no file
        is given.
        """
        native = _MegBuilder(self._version)
        for path, src in self._entries.items():
            if isinstance(src, bytes) or isinstance(src, bytearray):
                src = io.BytesIO(src)
            native.insert(path, src)
        if self._key is not None:
            native.set_encryption(self._key, self._iv)
        if outfile is not None:
            native.build(outfile)
        else:
            with io.BytesIO() as out:
                native.build(out)
                return out.getvalue()

    def __repr__(self):
        key_str = ''
        if self._key is not None:
            key_str = f', key={self._key!r}, iv={self._iv!r}'
        return f'MegBuilder({self._version!r}{key_str}, entries={self._entries!r})'
