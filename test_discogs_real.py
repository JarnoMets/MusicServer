#!/usr/bin/env python3
"""
Improved Discogs test harness that mirrors backend search logic (release_title, track, raw q, cleaned q, master),
prints URLs, status codes, number of results and first-result JSON. If the first result lacks a year, it will
attempt to fetch the `resource_url` to extract `year` or `released` date.

Usage: python3 test_discogs_real.py

This script reads token from ./discogs_token (trimmed) if present.
"""

import requests
import re
import json
import os
from urllib.parse import urlencode

BASE = "https://api.discogs.com/database/search"
TOKEN_PATHS = ["discogs_token", "./discogs_token", "../discogs_token"]


def load_token():
    # Try environment first
    t = os.environ.get("DISCOGS_TOKEN")
    if t:
        return t.strip()
    for p in TOKEN_PATHS:
        try:
            with open(p, "r") as f:
                v = f.read().strip()
                if v:
                    return v
        except Exception:
            continue
    return None


def clean_input(s: str) -> str:
    # Remove parenthetical remix/feat tags but preserve artist ampersands
    res = s
    # Remove parentheses/brackets that include keywords (remix, feat, etc.)
    res = re.sub(r"(?i)\s*[\(\[].*(feat|ft|with|prod|remix|edit|original|mix|extended|radio|vip|vocal|instrumental|version|official).*?[\)\]]", "", res)
    # Remove feat/ft/... trailing when not bracketed
    res = re.sub(r"(?i)\s+(feat|ft|with|prod)\.?\s+.*$", "", res)
    return res.strip()


def build_url(base, params, token=None):
    qp = params.copy()
    if token:
        qp['token'] = token
    return base + "?" + urlencode(qp)


def fetch(url, headers=None):
    try:
        r = requests.get(url, headers=headers or {})
        return r
    except Exception as e:
        print("  Exception while fetching:", e)
        return None


def try_tiers(artist, title, token=None):
    headers = {"User-Agent": "MusicServerTest/1.0"}
    clean_artist = clean_input(artist)
    clean_title = clean_input(title)
    raw_artist = artist
    raw_title = title

    tiers = []
    # Tier 1: release_title + artist (type=release)
    tiers.append({"type": "release", "release_title": clean_title, "artist": clean_artist})
    # Tier 2: track + artist (type=release)
    tiers.append({"type": "release", "track": clean_title, "artist": clean_artist})
    # Tier 3: raw combined query, type=release
    tiers.append({"type": "release", "q": f"{raw_artist} {raw_title}"})
    # Tier 4: cleaned combined query, no explicit type
    tiers.append({"q": f"{clean_artist} {clean_title}"})
    # Tier 5: master type search
    tiers.append({"type": "master", "q": f"{clean_artist} {clean_title}"})

    for idx, params in enumerate(tiers, 1):
        url = build_url(BASE, params, token)
        print(f"\nTier {idx} URL: {url}")
        r = fetch(url, headers)
        if r is None:
            print("  No response")
            continue
        print(f"  Status: {r.status_code}")
        if r.status_code != 200:
            print("  Body:", r.text[:1000])
            continue
        try:
            j = r.json()
        except Exception as e:
            print("  JSON parse error:", e)
            continue
        results = j.get('results', [])
        print(f"  Results: {len(results)}")
        if len(results) > 0:
            first = results[0]
            print("  First result title:", first.get('title'))
            print("  First result raw:")
            print(json.dumps(first, indent=2))
            # If year missing, try resource_url
            year = None
            y = first.get('year')
            if y:
                year = y
            else:
                res_url = first.get('resource_url')
                if res_url:
                    # append token if present
                    if token:
                        if '?' in res_url:
                            res_url = res_url + '&token=' + token
                        else:
                            res_url = res_url + '?token=' + token
                    print("  Fetching resource_url:", res_url)
                    rr = fetch(res_url, headers)
                    if rr and rr.status_code == 200:
                        try:
                            rd = rr.json()
                            print("  Resource JSON (excerpt):")
                            print(json.dumps({k: rd.get(k) for k in ['year','released','id','title']}, indent=2))
                            if rd.get('year'):
                                year = rd.get('year')
                            elif rd.get('released'):
                                year = rd.get('released').split('-')[0]
                        except Exception as e:
                            print("  Resource JSON parse error:", e)
                    else:
                        if rr:
                            print("  Resource fetch status:", rr.status_code, rr.text[:400])
            print("  Effective year:", year)
            return True
        else:
            # continue to next tier
            continue
    print("  No matches found in any tier")
    return False


if __name__ == '__main__':
    token = load_token()
    print("Using token:", bool(token))

    cases = [
        ("Turno", "1989"),
        ("Ivy Lab", "20 Questions"),
        ("Conway the Machine", "200 Pies (ft. 2Chainz)"),
        ("Disrupta & Furniss", "3 seconds"),
        ("Dillinja", "30Hz (L-Side Remix)"),
        ("1991", "Jungle Cats"),
        ("The Epiphany feat. Matt Wilson", "Chant (Extended Mix)")
    ]

    for artist, title in cases:
        print('\n' + '='*80)
        print(f"Testing: {artist} - {title}")
        try_tiers(artist, title, token)
import requests
import sys

TOKEN = "gfppetqSartEsxiKrpMbAckgwNeJUjrgpfazLXEM"
BASE_URL = "https://api.discogs.com/database/search"

def test_case(artist, title):
    print(f"\n--- Testing: {artist} - {title} ---")
    
    # Simulate cleaning
    def clean(s):
        import re
        res = s
        # Lighter cleaning for testing
        res = re.sub(r"(?i)\s*[\(\[].*(feat|ft|with|prod|remix|edit|original|mix|extended|radio|vip|vocal|instrumental|version|official).*[\)\]]", "", res)
        res = re.sub(r"(?i)\s+(feat|ft|with|prod)\.?\s+.*$", "", res)
        return res.strip()

    c_artist = clean(artist)
    c_title = clean(title)
    print(f"Cleaned: {c_artist} - {c_title}")

    # Tiers
    tiers = [
        {"type": "release", "release_title": c_title, "artist": c_artist},
        {"type": "release", "track": c_title, "artist": c_artist},
        {"type": "release", "q": f"{c_artist} {c_title}"},
        {"q": f"{c_artist} {c_title}"}
    ]

    for i, params in enumerate(tiers, 1):
        params["token"] = TOKEN
        headers = {"User-Agent": "MusicServerTest/1.0"}
        try:
            resp = requests.get(BASE_URL, params=params, headers=headers)
            if resp.status_code == 200:
                data = resp.json()
                results = data.get("results", [])
                if results:
                    first = results[0]
                    print(f"Tier {i} Success: {first.get('title')} ({first.get('year')})")
                    return
                else:
                    print(f"Tier {i} No results")
            else:
                print(f"Tier {i} Error: {resp.status_code} {resp.text}")
        except Exception as e:
            print(f"Tier {i} Exception: {e}")

test_cases = [
    ("Turno", "1989"),
    ("Ivy Lab", "20 Questions"),
    ("Conway the Machine", "200 Pies (ft. 2Chainz)"),
    ("Disrupta & Furniss", "3 seconds"),
    ("Dillinja", "30Hz (L-Side Remix)"),
    ("1991", "Jungle Cats")
]

for a, t in test_cases:
    test_case(a, t)
