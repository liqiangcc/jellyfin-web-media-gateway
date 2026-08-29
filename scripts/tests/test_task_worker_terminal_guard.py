import importlib.util, pathlib, sys, unittest
ROOT=pathlib.Path(__file__).resolve().parents[2]; PATH=ROOT/'scripts'/'task-worker-terminal-guard.py'
spec=importlib.util.spec_from_file_location('task_worker_terminal_guard',PATH); mod=importlib.util.module_from_spec(spec); sys.modules[spec.name]=mod; spec.loader.exec_module(mod)
def snap(state='open',labels=None,comments=None,owner='alice',attempt=2): return {'state':state,'labels':labels or ['env:cloud','status:in-progress'],'comments':comments or [],'owner':owner,'attempt':attempt}
class Tests(unittest.TestCase):
 def d(self,s): return mod.evaluate(s,expected_attempt=2,expected_owner='alice')
 def test_authorized_execution(self): self.assertTrue(self.d(snap()).allowed)
 def test_authorized_blocker(self): self.assertEqual(self.d(snap()).reason,'authorized')
 def test_closed(self): self.assertEqual(self.d(snap(state='closed')).reason,'issue-not-open')
 def test_done(self): self.assertEqual(self.d(snap(labels=['status:done'])).reason,'status-done')
 def test_final_acceptance(self): self.assertEqual(self.d(snap(comments=['[FINAL ACCEPTANCE]'])).reason,'final-acceptance-present')
 def test_owner_mismatch(self): self.assertEqual(self.d(snap(owner='bob')).reason,'owner-mismatch')
 def test_attempt_superseded(self): self.assertEqual(self.d(snap(attempt=3)).reason,'attempt-superseded')
 def test_not_in_progress(self): self.assertEqual(self.d(snap(labels=['status:review'])).reason,'not-in-progress')
 def test_rejection_is_pure(self):
  s=snap(comments=['[FINAL ACCEPTANCE]']); before=repr(s); self.assertFalse(self.d(s).allowed); self.assertEqual(repr(s),before)
if __name__=='__main__': unittest.main()
