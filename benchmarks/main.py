from io import BytesIO

import pyperf
from werkzeug.formparser import MultiPartParser

from benchmarks.sanic_multipart import sanic_multipart

werkzeug_multipart = MultiPartParser()

body = (
    # data
    b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\n"
    b'Content-Disposition: form-data; name="field0"\r\n\r\n'
    b"value0\r\n"
    # file
    b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\n"
    b'Content-Disposition: form-data; name="file"; filename="file.txt"\r\n'
    b"Content-Type: text/plain\r\n\r\n"
    b"<file content>\r\n"
    # data
    b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\n"
    b'Content-Disposition: form-data; name="field1"\r\n\r\n'
    b"value1\r\n"
    b"--a7f7ac8d4e2e437c877bb7b8d7cc549c--\r\n"
)
boundary = b"a7f7ac8d4e2e437c877bb7b8d7cc549c"


def bench_multipart(runner: pyperf.Runner):
    runner.bench_func(
        "sanic multipart parser",
        lambda: sanic_multipart(body, boundary),
    )
    runner.bench_func(
        "werkzeug multipart parser",
        lambda: werkzeug_multipart.parse(BytesIO(body), boundary, None),
    )


if __name__ == "__main__":
    runner = pyperf.Runner()

    bench_multipart(runner)
