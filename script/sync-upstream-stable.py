#!/usr/bin/env python3
"""Check upstream stable releases, or merge one into a clean stable-based branch."""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PIN = ROOT / '.github/upstream-stable.json'
REPOSITORY = 'warpdotdev/warp'
STABLE_TAG = re.compile(r'v\d+\.\d{4}\.\d{2}\.\d{2}\.\d{2}\.\d{2}\.stable_\d+')


def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def latest_stable(pages):
    releases = [r for page in pages for r in page
                if not r.get('draft', True) and not r.get('prerelease', True)
                and STABLE_TAG.fullmatch(r.get('tag_name', ''))]
    if not releases:
        raise RuntimeError('No published stable application release found.')
    return max(releases, key=lambda r: (r['published_at'], r['tag_name']))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--merge', action='store_true', help='Merge the latest stable release locally; never push.')
    args = parser.parse_args()
    pages = json.loads(run('gh', 'api', '--paginate', '--slurp',
                           f'repos/{REPOSITORY}/releases?per_page=100'))
    release = latest_stable(pages)
    tag = release['tag_name']
    pin = json.loads(PIN.read_text())
    print(f"Pinned: {pin['tag']}\nLatest stable: {tag}")
    if not args.merge:
        return
    if run('git', 'status', '--porcelain'):
        raise RuntimeError('Commit or stash local changes before merging stable.')
    run('git', 'symbolic-ref', '--quiet', '--short', 'HEAD')
    run('git', 'merge-base', '--is-ancestor', pin['commit'], 'HEAD')
    run('git', 'fetch', '--no-tags', f'https://github.com/{REPOSITORY}.git',
        f'refs/tags/{tag}:refs/fork-upstream/stable-candidate')
    commit = run('git', 'rev-parse', 'refs/fork-upstream/stable-candidate^{commit}')
    if tag == pin['tag']:
        if commit != pin['commit']:
            raise RuntimeError('The pinned upstream tag moved; refusing to merge.')
        print('Already using the latest stable release.')
        return
    run('git', 'merge-base', '--is-ancestor', pin['commit'], commit)
    try:
        run('git', 'merge', '--no-ff', '--no-commit', commit)
    except subprocess.CalledProcessError:
        subprocess.run(['git', 'merge', '--abort'], cwd=ROOT, check=False)
        raise RuntimeError('Stable merge conflicted and was aborted. Resolve in a separate worktree.') from None
    PIN.write_text(json.dumps({'repository': REPOSITORY, 'tag': tag, 'commit': commit}, indent=2) + '\n')
    run('git', 'add', '.github/upstream-stable.json')
    run('git', 'commit', '-m', f'Merge upstream stable {tag}')
    print('Merged locally. Run checks and push when ready.')


if __name__ == '__main__':
    main()
