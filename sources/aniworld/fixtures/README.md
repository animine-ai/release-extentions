# EP04 AniWorld fixtures

These small HTML files are independently authored, minimal structural reconstructions of the public page grammar observed in the raw captures listed by `../provenance-fixtures.json`. They retain only the element grouping, selectors, and factual language/episode shapes needed by deterministic tests. Titles, slugs, episode values, and redirect IDs in the fixtures are synthetic. The positive recent and calendar pages retain their confirmed `Neue Episoden` and `Animekalender` headings; the postponement block retains the article heading and line-break-separated entry grammar.

The fixtures do not contain raw source HTML, provider descriptions, images, advertising, trackers, cookies, embedded media, or third-party scripts. The raw captures are evidence used during authoring and are not checked into this repository. AniWorld and other rightsholders retain all rights in their materials; this repository does not claim or grant a license to those materials.

`recent.html` and `calendar.html` each include one synthetic DE_SUB item and one DE_DUB item. `series.html` and `episode.html` use only the synthetic key `fixture-series`. The redirect paths in `episode.html` are synthetic numeric placeholders. `postponement.html` contains synthetic, unbound schedule facts and intentionally has no provider series identity or link.

The targeted negative fixtures are synthetic mutations of those reconstructed shapes: `unknown-track.html` has an unrecognized flag, `duplicate-row.html` repeats a recent row, `malformed-date.html` has an impossible date, `truncated.html` ends inside an open row, `bot.html` is a generic challenge response, and `missing-episode.html` omits episode metadata. None is a live capture.
