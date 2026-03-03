import requests
import json

base_url = "https://music.jarnomets.com/api"
token = "Baske123"
headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}

mappings = [
    # Drum & Bass
    ("drum and bass", "50945048-3555-414a-bc37-62610321a0d3"),
    ("d&b", "50945048-3555-414a-bc37-62610321a0d3"),
    ("DnB", "50945048-3555-414a-bc37-62610321a0d3"),
    ("DNB", "50945048-3555-414a-bc37-62610321a0d3"),
    ("jungle", "50945048-3555-414a-bc37-62610321a0d3"),
    
    # Hip Hop
    ("hiphop", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("hip hop", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("gangsta rap", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("rap", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("grime", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("trap", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("conscious hip hop", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    ("east coast hip hop", "4b0fc33d-956a-49a3-9b6b-1a4e4da84bbe"),
    
    # Rock
    ("alternative rock", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("rock", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("hard rock", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("nu metal", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("post-punk", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("progressive rock", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("psychedelic rock", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("new wave", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("ostrock", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    ("death metal", "1e5383bc-c9c2-4518-8ee3-9fc6c28a902b"),
    
    # House
    ("house", "4d893edb-659b-459a-bbaf-da5034ef15fa"),
    ("deep house", "4d893edb-659b-459a-bbaf-da5034ef15fa"),
    ("bass house", "4d893edb-659b-459a-bbaf-da5034ef15fa"),
    
    # Electronic
    ("electronic", "f253ab1f-2e59-41b4-9419-58d94375080e"),
    ("rave", "f253ab1f-2e59-41b4-9419-58d94375080e"),
    ("electroclash", "f253ab1f-2e59-41b4-9419-58d94375080e"),
    ("eurodance", "f253ab1f-2e59-41b4-9419-58d94375080e"),
    ("ambient", "f253ab1f-2e59-41b4-9419-58d94375080e"),
    
    # Breakbeat
    ("Break", "da23ceb8-4979-4c4a-95db-a9e475c53e29"),
    ("acid techno", "f1d81fa9-8b60-4409-b06a-a737c23287e2"), # Techno
    
    # UK Garage
    ("uk garage", "be91d58a-f5e2-48e5-9a9b-f36c394c370a"),
    ("uk bass, bass music", "be91d58a-f5e2-48e5-9a9b-f36c394c370a"),
]

def apply_mappings():
    for alias, genre_id in mappings:
        print(f"Mapping '{alias}' to {genre_id}...")
        resp = requests.post(f"{base_url}/admin/genres/aliases/backfill", 
                             headers=headers, 
                             json={"alias": alias, "genre_id": genre_id})
        if resp.status_code == 200:
            print(f"  Successfully mapped and backfilled: {resp.json()}")
        else:
            print(f"  Failed: {resp.status_code} {resp.text}")

if __name__ == "__main__":
    apply_mappings()
