import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('sync_stable', Path(__file__).with_name('sync-upstream-stable.py'))
sync = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sync)
OLD = 'v0.2026.05.27.09.22.stable_00'
NEW = 'v0.2026.06.03.09.49.stable_00'


def release(tag, date='2026-06-03T09:49:25Z', **kwargs):
    return dict(tag_name=tag, published_at=date, draft=False, prerelease=False, **kwargs)


class StableReleaseTests(unittest.TestCase):
    def test_excludes_preview_dev_drafts_and_unrelated_releases(self):
        entries = [release(NEW), release('v0.2026.06.04.09.49.preview_00'),
                   release('v0.2026.06.04.09.49.dev_00'), release('tui-screenshots-app5029')]
        draft = release('v0.2026.06.05.09.49.stable_00'); draft['draft'] = True
        prerelease = release('v0.2026.06.06.09.49.stable_00'); prerelease['prerelease'] = True
        self.assertEqual(sync.latest_stable([entries, [draft, prerelease]])['tag_name'], NEW)

    def test_pagination_and_publish_order(self):
        self.assertEqual(sync.latest_stable([[release(OLD, '2026-05-27')], [release(NEW)]])['tag_name'], NEW)

    def test_no_stable_fails_closed(self):
        with self.assertRaises(RuntimeError):
            sync.latest_stable([[release('v0.2026.06.04.09.49.dev_00')]])


class MergeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git('init', '-b', 'stable')
        self.git('config', 'user.name', 'Test')
        self.git('config', 'user.email', 'test@example.invalid')
        (self.root / 'base').write_text('base\n')
        self.commit('base')
        self.base = self.git('rev-parse', 'HEAD')
        self.git('tag', OLD)
        self.git('checkout', '-b', 'upstream')
        (self.root / 'base').write_text('upstream\n')
        self.commit('upstream stable')
        self.git('tag', NEW)
        self.git('checkout', 'stable')
        (self.root / '.github').mkdir()
        self.pin = self.root / '.github/upstream-stable.json'
        self.pin.write_text(json.dumps(dict(repository=sync.REPOSITORY, tag=OLD, commit=self.base)))
        (self.root / 'dither').write_text('custom effect\n')
        self.commit('fork controls')
        self.before = self.git('rev-parse', 'HEAD')
        self.patches = [patch.object(sync, 'ROOT', self.root), patch.object(sync, 'PIN', self.pin),
                        patch('sys.argv', ['sync-upstream-stable.py', '--merge'])]
        for p in self.patches:
            p.start(); self.addCleanup(p.stop)
        original_run = sync.run
        def local_run(*args):
            if args[:2] == ('gh', 'api'):
                return json.dumps([[release(NEW)]])
            if args[:2] == ('git', 'fetch'):
                self.git('update-ref', 'refs/fork-upstream/stable-candidate', NEW)
                return ''
            return original_run(*args)
        p = patch.object(sync, 'run', local_run); p.start(); self.addCleanup(p.stop)

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, text=True, stderr=subprocess.DEVNULL).strip()

    def commit(self, message):
        self.git('add', '.'); self.git('commit', '-m', message)

    def test_merge_preserves_custom_changes_and_updates_pin(self):
        sync.main()
        self.assertEqual((self.root / 'dither').read_text(), 'custom effect\n')
        self.assertEqual((self.root / 'base').read_text(), 'upstream\n')
        self.assertEqual(json.loads(self.pin.read_text())['tag'], NEW)
        self.assertEqual(len(self.git('rev-list', '--parents', '-n', '1', 'HEAD').split()), 3)
        sync.main()  # Same immutable tag is a no-op.

    def test_dirty_tree_refused(self):
        (self.root / 'dither').write_text('unsaved\n')
        with self.assertRaisesRegex(RuntimeError, 'Commit or stash'):
            sync.main()
        self.assertEqual(self.git('rev-parse', 'HEAD'), self.before)

    def test_conflict_aborts_without_losing_fork(self):
        (self.root / 'base').write_text('fork change\n')
        self.commit('fork conflict')
        before = self.git('rev-parse', 'HEAD')
        with self.assertRaisesRegex(RuntimeError, 'conflicted'):
            sync.main()
        self.assertEqual(self.git('rev-parse', 'HEAD'), before)
        self.assertEqual(self.git('status', '--porcelain'), '')
        self.assertEqual((self.root / 'base').read_text(), 'fork change\n')


if __name__ == '__main__':
    unittest.main()
