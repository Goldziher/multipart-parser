from io import BytesIO

import pyperf
from fast_multipart_parser import parse_multipart_form_data
from sanic.request import parse_multipart_form as sanic_multipart
from werkzeug.formparser import MultiPartParser

werkzeug_multipart = MultiPartParser()

random_data = b"abc" * 4096

body = b"""
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="_id"

637ca2c6a8178b1d6aab4140
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="index"

0
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="guid"

92d50031-11ee-4756-af59-cd47a45082e7
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="isActive"

false
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="balance"

$2,627.33
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="picture"

http://placehold.it/32x32
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="age"

36
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="eyeColor"

blue
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="name"

Colette Suarez
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="gender"

female
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="company"

ZENTILITY
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="email"

colettesuarez@zentility.com
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="phone"

+1 (841) 509-2669
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="address"

400 Polar Street, Emory, Palau, 3376
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="about"

Deserunt nostrud quis enim fugiat labore labore sint deserunt aliquip est fugiat mollit commodo. Labore pariatur laboris ut irure voluptate aliqua non ex enim. Dolor ea mollit dolore anim eu velit labore aliquip laborum irure duis aliqua sunt sint. Ex elit ea irure nisi qui exercitation ullamco occaecat eu culpa magna quis dolor dolor. Officia nostrud consectetur exercitation consequat qui est dolore cillum dolor minim tempor.

--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="registered"

2015-12-11T05:34:25 -01:00
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="latitude"

-14.326509
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="longitude"

-32.417451
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="greeting"

Hello, Colette Suarez! You have 4 unread messages.
--b1f5d0f0e03874e20b4c5bd2851ec499
Content-Disposition: form-data; name="favoriteFruit"

banana
--b1f5d0f0e03874e20b4c5bd2851ec499--
"""

boundary = b"b1f5d0f0e03874e20b4c5bd2851ec499"


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
