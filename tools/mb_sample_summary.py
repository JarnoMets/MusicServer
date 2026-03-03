#!/usr/bin/env python3
from collections import Counter

fn='/tmp/mb_sample_results.tsv'
rows=[]
with open(fn,'r',encoding='utf-8') as f:
    header=f.readline()
    for line in f:
        parts=line.rstrip('\n').split('\t')
        if len(parts)<4:
            continue
        rows.append(parts)

total=len(rows)
counts=Counter()
real_tags=Counter()
for r in rows:
    tag=r[3].strip()
    counts[tag]+=1
    if tag and tag!='(none)' and tag!='(error)' and not tag.startswith('(mb_error'):
        real_tags[tag]+=1

print(f'Total_sampled={total}')
print('\nTag counts:')
for tag,c in counts.most_common():
    print(f'{c:3d}  {tag}')

print('\nNumber with a real tag (not (none)/(error)/(mb_error)): ', sum(real_tags.values()))
print('Number (none):', counts.get('(none)',0))
print('Number (error):', counts.get('(error)',0))
mb_errors = sum(c for t,c in counts.items() if t.startswith('(mb_error'))
print('Number (mb_error_*):', mb_errors)

print('\nDistinct real tags (top):')
for t,c in real_tags.most_common(50):
    print(f'{c:3d}  {t}')
