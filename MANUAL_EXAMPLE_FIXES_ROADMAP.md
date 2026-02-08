# Manual Example Fixes Roadmap

**Created**: February 8, 2026  
**Status**: Ready to Start  
**Location**: Phase 7.7 in plan.md

---

## Overview

After completing the core parser enhancement (fixing identifier grammar bug and validating curly brace support), **33 examples remain failing**:
- **26 SYNTAX_ERROR files** - Need curly brace syntax conversion
- **7 OTHER_ERROR files** - Need runtime issue fixes

**Total Estimated Time**: 8-12 hours

---

## Task Breakdown

### ✅ Completed: Parser Enhancement
- Fixed identifier grammar bug (reserved word collision)
- Validated curly brace syntax support
- Created test infrastructure
- **Current State**: 25/58 passing (43.1%)

---

### 📋 Phase 7.7 Tasks (Added to plan.md)

#### Task 1: Fix SYNTAX_ERROR Examples - CTF Category (6 files)
**Estimated Time**: 1.5-2 hours  
**Priority**: HIGH

**Files**:
1. `06_ctf_automation.talon`
2. `ctf_blind_rop.talon`
3. `ctf_kernel_exploit.talon`
4. `ctf_multi_stage_pwn.talon`
5. `ctf_one_gadget_pwn.talon`
6. `ctf_shellcode_encoder.talon`

---

#### Task 2: Fix SYNTAX_ERROR Examples - Exploit Chain Category (4 files)
**Estimated Time**: 1-1.5 hours  
**Priority**: HIGH

**Files**:
1. `exploit_chain_buffer_overflow.talon`
2. `exploit_chain_format_string.talon`
3. `exploit_chain_heap_uaf.talon`
4. `exploit_chain_with_recovery.talon`

---

#### Task 3: Fix SYNTAX_ERROR Examples - Orchestrator Category (4 files)
**Estimated Time**: 1-1.5 hours  
**Priority**: MEDIUM

**Files**:
1. `orchestrator_graph.talon`
2. `orchestrator_parallel.talon`
3. `orchestrator_resilient.talon`
4. `orchestrator_timetravel.talon`

---

#### Task 4: Fix SYNTAX_ERROR Examples - Swarm Category (4 files)
**Estimated Time**: 1-1.5 hours  
**Priority**: MEDIUM

**Files**:
1. `swarm_libc_leak.talon`
2. `swarm_mass_exploit.talon`
3. `swarm_mass_pwn.talon`
4. `swarm_subnet_scan.talon`

---

#### Task 5: Fix SYNTAX_ERROR Examples - Phase 2 Category (3 files)
**Estimated Time**: 1 hour  
**Priority**: MEDIUM

**Files**:
1. `phase21_meta_programming.talon`
2. `phase22_demo.talon`
3. `phase22_symbiotic_execution.talon`

---

#### Task 6: Fix SYNTAX_ERROR Examples - Miscellaneous Category (6 files)
**Estimated Time**: 1.5-2 hours  
**Priority**: MEDIUM

**Files**:
1. `memory_scrubbing.talon`
2. `polymorphic_shellcode.talon` (if still SYNTAX_ERROR)
3. `time_travel_debugging.talon`
4. `tutorial_01_basics.talon`
5. `tutorial_02_exploitation.talon`
6. `world_class_exploit.talon`

---

#### Task 7: Fix OTHER_ERROR Examples - Runtime Issues (7 files)
**Estimated Time**: 2-4 hours  
**Priority**: HIGH

**Files**:
1. `ctf_heap_tcache_poison.talon`
2. `ctf_quick_exploitation.talon`
3. `ctf_ret2libc_pwn.talon`
4. `polymorphic_shellcode.talon` (if still OTHER_ERROR)
5. `rop_dsl_showcase.talon`
6. `ssh_exploitation.talon`
7. `swarm_agent_deployment.talon`

---

## Common Fix Patterns (SYNTAX_ERROR)

### Pattern 1: If Statement with Code on Same Line
```talon
# BROKEN:
if condition {    stmt1    stmt2
}

# FIXED:
if condition {
    stmt1
    stmt2
}
```

### Pattern 2: Else Block on Same Line
```talon
# BROKEN:
} else {    stmt
}

# FIXED:
} else {
    stmt
}
```

### Pattern 3: Try/Catch with End Keyword
```talon
# BROKEN:
try
    stmt
catch e
    stmt
end

# FIXED:
try {
    stmt
} catch e {
    stmt
}
```

### Pattern 4: Nested If/Else Without Braces
```talon
# BROKEN:
if outer
    if inner
        stmt
    else
        stmt
    end
else
    stmt
end

# FIXED:
if outer {
    if inner {
        stmt
    } else {
        stmt
    }
} else {
    stmt
}
```

### Pattern 5: Function Definition with Code on Same Line
```talon
# BROKEN:
define function foo() {    stmt
}

# FIXED:
define function foo() {
    stmt
}
```

---

## Manual Fix Workflow (Per File)

### Step 1: Read File
```
Read(examples/<filename>.talon)
```

### Step 2: Identify Issues
Look for:
- If/else blocks with code on same line
- Try/catch with `end` keyword
- Nested if/else without curly braces
- Malformed brace placement

### Step 3: Fix Line by Line
Use Edit tool to fix each issue:
```
Edit(file_path, old_string, new_string)
```

### Step 4: Test
```bash
powershell -Command ".\target\debug\talon.exe run examples\<filename>.talon --dry-run 2>&1"
```

### Step 5: Verify
- Exit code should be 0
- No SYNTAX_ERROR in output
- File parses correctly

### Step 6: Move to Next File
Repeat for all files in category

---

## Runtime Error Investigation (OTHER_ERROR)

### Investigation Steps

1. **Run file and capture error**:
```bash
talon run --dry-run <file> 2>&1 > error_<file>.txt
```

2. **Categorize error type**:
- **Missing builtin function** → Add to `src/interpreter.rs`
- **Type mismatch** → Fix type conversion
- **Undefined variable** → Check if builtin exists, add if needed
- **Stack overflow** → Simplify nested expressions
- **Other runtime error** → Deep investigation

3. **Fix based on error type**:
- For missing builtins: Add to interpreter match statement
- For type issues: Add proper type conversion
- For stack overflow: Refactor complex expressions
- For other errors: Case-by-case analysis

4. **Re-test until clean**:
```bash
talon run --dry-run <file>
# Should exit with code 0, no errors
```

---

## Expected Fixes for OTHER_ERROR Files

Based on earlier analysis:

1. **ctf_heap_tcache_poison.talon**:
   - Likely missing: `allocate()`, `edit()` builtins
   - May need: Heap-specific map properties

2. **ctf_quick_exploitation.talon**:
   - Likely missing: Quick exploitation helper functions
   - May need: Auto-exploit builtins

3. **ctf_ret2libc_pwn.talon**:
   - Likely issue: `libc.strings` property (already fixed in interpreter)
   - May need: Ret2libc-specific helpers

4. **polymorphic_shellcode.talon**:
   - Already has polymorphic engine implementation
   - May need: Integration fixes or API mismatches

5. **rop_dsl_showcase.talon**:
   - Likely issue: ROP chain method calls
   - May need: Additional ROP helper methods

6. **ssh_exploitation.talon**:
   - Already has SSH bridge implementation
   - May need: SSH method fixes or property access

7. **swarm_agent_deployment.talon**:
   - Already has swarm infrastructure
   - May need: Agent deployment method fixes

---

## Progress Tracking

After completing each task:
1. Mark task as [x] in plan.md
2. Run full test suite: `powershell -File scripts/test_all_examples.ps1`
3. Update pass count
4. Document any new issues discovered

---

## Success Criteria

**Target**: 58/58 examples passing (100%)

**After All Tasks Complete**:
- ✅ All 26 SYNTAX_ERROR files converted to curly brace syntax
- ✅ All 7 OTHER_ERROR files have runtime issues resolved
- ✅ Full test suite shows 58/58 passing
- ✅ Zero SYNTAX_ERROR examples
- ✅ Zero OTHER_ERROR examples
- ✅ Ready for production audit

---

## Next Steps

1. Start with **Task 1: Fix SYNTAX_ERROR Examples - CTF Category**
2. Complete tasks sequentially (dependencies in place)
3. After all Phase 7.7 tasks done, run **Final Example Validation**
4. Proceed to **Production Code Quality Audit**

---

## References

- **plan.md** - Phase 7.7 tasks (lines 5806-6021)
- **PARSER_ENHANCEMENT_FINAL_STATUS.md** - Parser work completed
- **test_after_restore.txt** - Current test results (25/58 passing)
- **scripts/test_all_examples.ps1** - Automated validation script
