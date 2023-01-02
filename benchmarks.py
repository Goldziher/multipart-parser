from io import BytesIO
from os import urandom

import pyperf
from fast_multipart_parser import parse_multipart_form_data
from sanic.request import parse_multipart_form as sanic_multipart
from werkzeug.formparser import MultiPartParser

werkzeug_multipart = MultiPartParser()

random_data = urandom(4096)

body = (
    (
        # data
        b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\n"
        b'Content-Disposition: form-data; name="field0"\r\n\r\n'
        b"value0\r\n"
        # data
        b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\n"
        b'Content-Disposition: form-data; name="field1"\r\n\r\n'
        b"value1\r\n"
        b"--a7f7ac8d4e2e437c877bb7b8d7cc549c--\r\n"
        # file
        b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\n"
        b'Content-Disposition: form-data; name="file1"; filename="file.txt"\r\n'
        b"Content-Type: text/plain\r\n\r\n"
    )
    + random_data
    + b"\r\n"
)

boundary = b"a7f7ac8d4e2e437c877bb7b8d7cc549c"


def bench_multipart(runner: pyperf.Runner):
    runner.bench_func(
        "sanic.request.parse_multipart_form",
        lambda: sanic_multipart(body, boundary),
    )
    runner.bench_func(
        "werkzeug.MultiPartParser.parse",
        lambda: werkzeug_multipart.parse(BytesIO(body), boundary, None),
    )
    runner.bench_func(
        "fast_multipart_parser.parse_multipart_form_data",
        lambda: parse_multipart_form_data(body, boundary, b"UTF-8"),
    )


if __name__ == "__main__":
    runner = pyperf.Runner()

    bench_multipart(runner)
