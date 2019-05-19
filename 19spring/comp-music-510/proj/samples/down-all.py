from requests_html import HTML
from requests_futures.sessions import FuturesSession

BASE = 'http://theremin.music.uiowa.edu/'

# Downloaded from `MISPiano.html`
with open('mispiano.html') as f:
    root = HTML(html=f.read())

mfs = [h for h in root.links if 'Piano.mf' in h]
nmfs = len(mfs)
print(f'total: {nmfs} files')

session = FuturesSession(max_workers=10)

ars = [session.request('get', BASE + h) for h in mfs]
for i, ar in enumerate(ars):
    r = ar.result()
    fname = r.url.split('/')[-1]

    if r.status_code != 200:
        print(f'Failed to download {fname}')
        continue

    with open(fname, 'wb') as f:
        f.write(r.content)

    print(f'[{i+1}/{nmfs}] {fname} done')
