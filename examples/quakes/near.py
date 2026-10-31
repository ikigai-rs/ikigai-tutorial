"""How far each earthquake of a week was from a point you name: a small Python peer.

"When a rule is not enough", in "Exploring without Rust", mounts this into the reader's
`ikigai` and pipes a recorded week into it:

    python3 near.py /tmp/quakes-near.sock
    ikigai --files . --mount urn:quakes-py:=/tmp/quakes-near.sock \
        -c 'source urn:file:quakes-2024-01-01.geojson | urn:quakes-py:near:35.68:139.69'

The peer reads no file and asks the kernel for nothing: the week arrives as its input, and
the answer goes back. Needs ``ikigai-python`` (``pip install <path to ikigai-python>``),
which brings its one dependency, ``blake3``.
"""

from __future__ import annotations

import json
import math
import sys
from typing import Annotated

from ikigai import InvalidArgumentError, endpoint, serve

# ANCHOR: near
EARTH_KM = 6371.0  # the mean radius of the Earth


def km(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """The great-circle distance between two points, by the haversine formula."""
    p1, p2 = math.radians(lat1), math.radians(lat2)
    dp, dl = p2 - p1, math.radians(lon2 - lon1)
    a = math.sin(dp / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return 2 * EARTH_KM * math.asin(math.sqrt(a))


# A family: `{lat}` and `{lon}` come from the name, the week from the pipe. A pure function
# of the three, so it says `cacheable=True`; whether the mounting host keeps a copy is the
# host's call, and the chapter shows what ikigai-cli does with it.
@endpoint("urn:quakes-py:near:{lat}:{lon}", output="text/csv", cacheable=True,
          summary="The five earthquakes of a GeoJSON week nearest a point, nearest first")
def near(lat: Annotated[float, "the point's latitude, in degrees north"],
         lon: Annotated[float, "the point's longitude, in degrees east"],
         week: Annotated[str, "a USGS GeoJSON feed, piped in"]) -> str:
    if not -90 <= lat <= 90:
        raise InvalidArgumentError("lat", f"{lat} is not between -90 and 90")
    if not -180 <= lon <= 180:
        raise InvalidArgumentError("lon", f"{lon} is not between -180 and 180")
    rows = []
    for quake in json.loads(week)["features"]:
        qlon, qlat = quake["geometry"]["coordinates"][:2]
        rows.append((km(lat, lon, qlat, qlon), quake["properties"]["mag"],
                     quake["properties"]["place"]))
    rows.sort()
    lines = ["km,mag,place"]
    lines += [f'{d:.0f},{mag},"{place}"' for d, mag, place in rows[:5]]
    return "\n".join(lines)
# ANCHOR_END: near


if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/quakes-near.sock"
    print(f"serving urn:quakes-py:near:{{lat}}:{{lon}} on {path}", file=sys.stderr)
    serve([near], path)
