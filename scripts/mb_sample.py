#!/usr/bin/env python3
"""
Sample 30 random tracks from the server API and query MusicBrainz recording search for top tags.
Outputs TSV: id\tartist\ttitle\tmusicbrainz_tag
"""
import json,random,urllib.parse,requests,time,sys

API_URL='https://music.jarnomets.com/api/music?limit=2000'
UA='MusicServerSampler/0.1 (jarno@jarnomets.com)'

try:
    r = requests.get(API_URL, headers={'User-Agent':UA}, timeout=20)
    r.raise_for_status()
    data = r.json()
except Exception as e:
    print('ERROR: failed to fetch music API:', e, file=sys.stderr)
    sys.exit(2)

# extract list
if isinstance(data, dict) and 'items' in data:
    items = data['items']
elif isinstance(data, list):
    items = data
else:
    # find first list value
    items = None
    for v in data.values():
        if isinstance(v, list):
            items = v
            break
    if items is None:
        print('ERROR: could not find track list in API response', file=sys.stderr)
        sys.exit(3)

if not items:
    print('No tracks found', file=sys.stderr)
    sys.exit(4)

random.seed(42)
sample = random.sample(items, min(30, len(items)))


def get_top_tag_from_mb(mb_json):
    recs = mb_json.get('recordings') or []
    tag_counts = {}
    for r in recs:
        for t in r.get('tags', []):
            name = t.get('name')
            count = t.get('count', 0) or 0
            if name:
                tag_counts[name] = tag_counts.get(name, 0) + count
    if not tag_counts:
        return None
    # return highest total count
    return sorted(tag_counts.items(), key=lambda x: -x[1])[0][0]

print('id\tartist\ttitle\tmusicbrainz_tag')
for tr in sample:
    tid = tr.get('id')
    artist = tr.get('artist') or ''
    title = tr.get('title') or ''
    # build query; prefer recording:"title" AND artist:"artist"
    q = f'recording:"{title}"'
    if artist:
        q += f' AND artist:"{artist}"'
    qenc = urllib.parse.quote(q)
    url = f'https://musicbrainz.org/ws/2/recording/?query={qenc}&fmt=json&limit=5'
    try:
        mr = requests.get(url, headers={'User-Agent':UA}, timeout=15)
        if mr.status_code != 200:
            tag = f'(mb_error_{mr.status_code})'
        else:
            mb = mr.json()
            tag = get_top_tag_from_mb(mb) or '(none)'
    except Exception:
        tag = '(error)'
    # sanitize tabs/newlines in artist/title/tag
    def s(x):
        return str(x).replace('\t',' ').replace('\n',' ').strip()
    print(f"{s(tid)}\t{s(artist)}\t{s(title)}\t{s(tag)}")
    time.sleep(1)
