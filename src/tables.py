import enum
from typing import List, Dict, Any, Sequence, Callable, Tuple


def main():
    table = Table(('A', 'Header with more words', 'c'), header_effect=Effect.underline)
    table.line(A=3.141, c=Effect.yellow('hello'))
    table.line(A=Effect.cyan(202.71828182), Header_with_more_words=Effect.red(1234567))
    table.line(a=Effect.bold(Effect.magenta(2.1)), c=Effect.blue(Effect.underline('blabla')))
    table.line(A=Effect.green('long string'), Header_with_more_words=32)
    table.line(a=Effect.green(12345), heaDer_wIth_MORe_woRDs='something')
    print(table)


class _Color(enum.Enum):
    RED = '\033[91m'
    GREEN = '\033[92m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    YELLOW = '\033[93m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    ENDC = '\033[0m'


class Effect:
    def __init__(self, color: _Color, value):
        self.color = color
        self.value = value

    @staticmethod
    def red(value: Any):
        return Effect(_Color.RED, value)

    @staticmethod
    def green(value: Any):
        return Effect(_Color.GREEN, value)

    @staticmethod
    def blue(value: Any):
        return Effect(_Color.BLUE, value)

    @staticmethod
    def magenta(value: Any):
        return Effect(_Color.MAGENTA, value)

    @staticmethod
    def cyan(value: Any):
        return Effect(_Color.CYAN, value)

    @staticmethod
    def yellow(value: Any):
        return Effect(_Color.YELLOW, value)

    @staticmethod
    def bold(value: Any):
        return Effect(_Color.BOLD, value)

    @staticmethod
    def underline(value: Any):
        return Effect(_Color.UNDERLINE, value)
    

class Table:
    def __init__(
            self, headers: Sequence[Any], float_precision: int = 3, column_space: int = 2,
            header_effect: Callable[[Any], Effect] | None = None, lower_keys: bool = True
    ):
        self._lower_keys = lower_keys
        if header_effect is not None:
            headers = [header_effect(h) for h in headers]
        self._string_formatter = StringFormatter(float_precision=float_precision)
        self._headers: Sequence[Tuple[str, int]] = [self._string_formatter(h, 0, 0) for h in headers]
        self._normed_headers: List[str] = [self._norm_header(h) for h in headers]
        self._lines: List[Dict[str, Any]] = []
        self._column_space = column_space

    def line(self, **kwargs):
        normed_args = {self._norm_header(k): v for k, v in kwargs.items()}
        for key in normed_args:
            if key not in self._normed_headers:
                raise KeyError(f'Add value for header "{key}", but this header does not exist. '
                               f'Valid headers are: {', '.join(self._normed_headers)}')
        self._lines.append(normed_args)

    def __repr__(self):
        column_width = {nh: h[1] for nh, h in zip(self._normed_headers, self._headers)}
        sep = ' ' * self._column_space
        longest_float = {nh: 0 for nh in self._normed_headers}
        longest_int = {nh: 0 for nh in self._normed_headers}
        for line in self._lines:
            for header, value in line.items():
                str_len = self._string_formatter(value, 0, 0)[1]
                column_width[header] = max(column_width[header], str_len)
                if Table.has_value_type(value, float):
                    longest_float[header] = max(longest_float[header], str_len)
                if Table.has_value_type(value, int):
                    longest_int[header] = max(longest_int[header], str_len)

        header_line = sep.join(
            Table.ljust(h, l, column_width[nh]) for nh, (h, l) in zip(self._normed_headers, self._headers)
        )
        lines = [header_line]

        for line in self._lines:
            str_line = []
            for header in self._normed_headers:
                col_width = column_width[header]
                if header in line:
                    value = line[header]
                    str_value, str_len = self._string_formatter(value, longest_float[header], longest_int[header])
                    str_line.append(Table.ljust(str_value, str_len, col_width))
                else:
                    str_line.append(' ' * col_width)
            lines.append(sep.join(str_line))
        return '\n'.join(lines)

    def _norm_header(self, header):
        header, _header_len = self._string_formatter(header, 0, 0, ignore_effects=True)
        header = header.replace(' ', '_')
        if self._lower_keys:
            header = header.lower()
        return header

    @staticmethod
    def ljust(value: str, current_length: int, length: int):
        return value + ' ' * (length - current_length)

    @staticmethod
    def has_value_type(value, t):
        while isinstance(value, Effect):
            value = value.value
        return isinstance(value, t)


class StringFormatter:
    def __init__(self, float_precision: int = 3):
        self._float_fmt_string = f'{{:<.{float_precision}f}}'
        self.formatters: Dict[Any, Callable[[Any, int, int, bool], Tuple[str, int]]] = {
            float: self._format_float,
            int: StringFormatter._format_int,
            Effect: self._format_effect
        }
        self._np = None
        try:
            import numpy as np
            self._np = np
        except ImportError:
            pass

    def __call__(self, value, longest_float_hint: int, longest_int_hint: int, ignore_effects=False):
        value = self._normalize_numpy_types(value)
        formatter = self.formatters.get(type(value))
        if formatter is not None:
            return formatter(value, longest_float_hint, longest_int_hint, ignore_effects)
        str_value = str(value)
        return str_value, len(str_value)

    def _normalize_numpy_types(self, value):
        if self._np is not None:
            if self._np.issubdtype(type(value), self._np.floating):
                return float(value)
            if self._np.issubdtype(type(value), self._np.integer):
                return int(value)
        return value

    def _format_float(self, value: float, longest_float_hint: int, _longest_int_hint: int, _ignore_effects: bool):
        str_value = self._float_fmt_string.format(value).rjust(longest_float_hint)
        return str_value, len(str_value)

    @staticmethod
    def _format_int(value: float, _longest_float_hint: int, longest_int_hint: int, _ignore_effects: bool):
        str_value = str(value).rjust(longest_int_hint)
        return str_value, len(str_value)

    def _format_effect(self, effect: Effect, longest_float_hint, longest_int_hint: int, ignore_effects: bool):
        str_content, len_content = self(effect.value, longest_float_hint, longest_int_hint, ignore_effects)
        if ignore_effects:
            return str_content, len_content
        return effect.color.value + str_content + _Color.ENDC.value, len_content


if __name__ == '__main__':
    main()
