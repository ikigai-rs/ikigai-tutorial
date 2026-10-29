# quakes

The recorded window the ikigai Book's part "Exploring without Rust" runs on, bound in the
page's kernel at `urn:file:quakes-2024-01-01.geojson`.

`quakes-2024-01-01.geojson` was recorded on 2026-10-28 (its `metadata.generated`,
1793172664000, is 07:31:04 UTC that day) from
<https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime=2024-01-01&endtime=2024-01-08&minmagnitude=4.5>:
178 events, M4.5 and up, the week from 2024-01-01 to 2024-01-08, the largest the M7.5 Noto
Peninsula earthquake of 2024-01-01 (`us6000m0xl`). It is the U.S. Geological Survey's data,
which is in the public domain; it is committed byte for byte as USGS served it
(SHA-256 `a3843376c14c9ba771aae28b10f39bfded843a91b38c03bb5a148077ca5d167d`, 125,765 bytes).

`quakes-2024-01-08.geojson`, the week after, bound at `urn:file:quakes-2024-01-08.geojson`
for "Composites, one layer at a time", was recorded on 2026-10-29 (`metadata.generated`
1793269913000, 10:31:53 UTC that day) from
<https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime=2024-01-08&endtime=2024-01-15&minmagnitude=4.5>:
105 events, the largest the M6.7 earthquake 93 km SE of Sarangani, Philippines (`us6000m2jp`).
Committed byte for byte as USGS served it (SHA-256
`0faea2846db68d21e1ebdcff714f10f9d135f2d1717d520f3af911da35efdc1c`, 74,885 bytes).

The query is USGS's FDSN event service, not one of its rolling summary feeds
(`…/feed/v1.0/summary/*.geojson`), because a summary feed covers the last hour, day, week or
month and is never the same twice. Both answer the same GeoJSON format.
