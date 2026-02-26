# Error Message Review: src/skills/error.rs

## Overview
Review of all error messages in the SkillsError enum for clarity, helpfulness, and user-friendliness.

**Date:** 2026-02-20
**Reviewer:** Worker 1 (Orchestrator - Verification Phase)
**Scope:** src/skills/error.rs

---

## Summary of Findings

**Total Error Variants:** 16
- Excellent: 5 (31%)
- Good: 8 (50%)
- Needs Improvement: 3 (19%)

**Overall Assessment:** The error messages are generally well-structured with clear documentation. Most errors provide adequate context, but a few could be enhanced with more actionable guidance.

---

## Detailed Review

### ✅ EXCELLENT (5 errors)

#### 1. NpxNotFound
**Display Message:**
```
npx is required for this command. Install Node.js from https://nodejs.org
```
**Rating:** EXCELLENT
- **Clarity:** Very clear - user knows exactly what's missing
- **Helpfulness:** Provides direct solution with URL
- **User-friendliness:** Excellent - action-oriented and straightforward
- **No changes needed**

---

#### 2. EmptySkillsList
**Display Message:**
```
Agent '{}' has an empty skills list. Either remove the 'skills' field or add skills.
```
**Rating:** EXCELLENT
- **Clarity:** Clear explanation of the issue
- **Helpfulness:** Provides two clear options for resolution
- **User-friendliness:** Very helpful and actionable
- **No changes needed**

---

#### 3. InvalidSkillsEntryFormat
**Display Message:**
```
Invalid skills entry '{}' for agent '{}': {}. Valid formats: owner/repo or owner/repo@skill-name
```
**Rating:** EXCELLENT
- **Clarity:** Clear identification of the problem
- **Helpfulness:** Provides valid format examples
- **User-friendliness:** Very informative with concrete examples
- **No changes needed**

---

#### 4. SkillNameCollision
**Display Message:**
```
Skill name collision: '{}' exists in both project and global skills. Using project-level.
```
**Rating:** EXCELLENT
- **Clarity:** Clearly explains the conflict
- **Helpfulness:** Tells user what will happen (project-level takes precedence)
- **User-friendliness:** Informative without being alarming (it's a warning)
- **No changes needed**

---

#### 5. SkillDirectoryNotFound (skill name variant)
**Display Message:**
```
Skill '{}' not found in project or global directory. Use 'switchboard skills installed' to list available skills.
```
**Rating:** EXCELLENT
- **Clarity:** Clear about where it searched
- **Helpfulness:** Provides specific command to help user find skills
- **User-friendliness:** Very actionable
- **No changes needed**

---

### ✅ GOOD (8 errors)

#### 6. NpxCommandFailed
**Display Message:**
```
npx skills command '{}' failed with exit code {}: {}
```
**Rating:** GOOD
- **Clarity:** Clear - shows command, exit code, and stderr
- **Helpfulness:** Provides all technical details needed for debugging
- **User-friendliness:** Somewhat technical (exit codes may confuse non-technical users)
- **Suggestion:** Consider adding: "Check your network connection or verify the skill name"
- **Priority:** Low

---

#### 7. MalformedSkillMetadata
**Display Message:**
```
Malformed SKILL.md for '{}': {} ({})
```
**Rating:** GOOD
- **Clarity:** Clear identification of the problem
- **Helpfulness:** Shows skill name, reason, and path
- **User-friendliness:** Adequate - could be more specific about how to fix
- **Suggestion:** Add example of valid frontmatter format or link to documentation
- **Priority:** Low

---

#### 8. NetworkUnavailable
**Display Message:**
```
Network unavailable during {}: {}
```
**Rating:** GOOD
- **Clarity:** Clear - explains what operation failed
- **Helpfulness:** Shows error details
- **User-friendliness:** Adequate, but lacks troubleshooting suggestions
- **Suggestion:** Add "Check your internet connection and try again"
- **Priority:** Low

---

#### 9. ContainerInstallFailed
**Display Message:**
```
Container skill install failed for agent '{}': '{}' exited with code {}: {}
```
**Rating:** GOOD
- **Clarity:** Very clear - provides agent name, skill source, exit code, and stderr
- **Helpfulness:** Excellent debugging information
- **User-friendliness:** Technical but thorough
- **Suggestion:** Could mention that this is a container-specific error
- **Priority:** Low

---

#### 10. IoError
**Display Message:**
```
IO error during {} on {}: {}
```
**Rating:** GOOD
- **Clarity:** Clear - shows operation, path, and error message
- **Helpfulness:** Provides all necessary information
- **User-friendliness:** Generic but informative
- **No changes needed** (IO errors are typically system-level and hard to provide specific guidance)

---

#### 11. MissingFrontmatter
**Display Message:**
```
Missing YAML frontmatter in SKILL.md file: {}
```
**Rating:** GOOD
- **Clarity:** Clear - tells exactly what's missing
- **Helpfulness:** Shows the problematic file path
- **User-friendliness:** Adequate, but doesn't show what frontmatter should look like
- **Suggestion:** Add example format: "Expected format: ---\nname: my-skill\n---"
- **Priority:** Low

---

#### 12. InvalidSkillDirectory
**Display Message:**
```
Invalid skill directory (no SKILL.md found): {}
```
**Rating:** GOOD
- **Clarity:** Clear - explains exactly why it's invalid
- **Helpfulness:** Shows the problematic directory
- **User-friendliness:** Good - direct and specific
- **No changes needed**

---

#### 13. FieldMissing
**Display Message:**
```
Required field '{}' missing from frontmatter in: {}
```
**Rating:** GOOD
- **Clarity:** Very clear - shows exactly which field is missing
- **Helpfulness:** Identifies the specific problem
- **User-friendliness:** Good - specific and actionable
- **No changes needed**

---

### ⚠️ NEEDS IMPROVEMENT (3 errors)

#### 14. SkillNotFound
**Display Message:**
```
Skill not found: {}
```
**Rating:** NEEDS IMPROVEMENT
- **Clarity:** Clear but minimal
- **Helpfulness:** Doesn't provide guidance on what to do next
- **User-friendliness:** Not very helpful - user is left guessing
- **Issues:**
  - Doesn't suggest checking the skill name for typos
  - Doesn't mention how to search for available skills
  - Doesn't provide a link to skills.sh registry
- **Suggested improvement:**
  ```
  Skill '{}' not found. Verify the skill name or use 'switchboard skills list' to search available skills at skills.sh.
  ```
- **Priority:** MEDIUM

---

#### 15. SkillsDirectoryNotFound (path variant)
**Display Message:**
```
Skills directory not found: {}
```
**Rating:** NEEDS IMPROVEMENT
- **Clarity:** Clear, but lacks context
- **Helpfulness:** Doesn't explain what this means or what to do
- **User-friendliness:** Minimal - user may be confused about whether this is an error
- **Issues:**
  - Doesn't explain this might be normal (no skills installed yet)
  - Doesn't suggest running `switchboard skills installed` first
  - No context about project vs. global directory
- **Suggested improvement:**
  ```
  Skills directory not found: {}. This is normal if no skills are installed yet. Run 'switchboard skills list' to search for skills.
  ```
  OR (if non-fatal):
  ```
  No skills directory found at {}. No skills are installed. Use 'switchboard skills install' to add skills.
  ```
- **Priority:** MEDIUM

---

#### 16. RemoveFailed
**Display Message:**
```
Failed to remove skill '{}': {}
```
**Rating:** NEEDS IMPROVEMENT
- **Clarity:** Clear about the failure
- **Helpfulness:** Shows the reason, but could be more specific
- **User-friendliness:** Minimal guidance for troubleshooting
- **Issues:**
  - Doesn't suggest checking file permissions
  - Doesn't mention if the skill might be in use
  - Doesn't suggest manual cleanup options
- **Suggested improvement:**
  ```
  Failed to remove skill '{}': {}. Common causes: file permissions, skill in use, or filesystem issues. Check permissions or delete manually.
  ```
- **Priority:** MEDIUM

---

## Additional Observations

### Duplicate Naming Concern
There are two similarly named error variants:
- `SkillsDirectoryNotFound { path }` (line 235) - Directory path not found
- `SkillDirectoryNotFound { skill_name }` (line 326) - Skill by name not found

While distinct in purpose, the similar names could be confusing. Consider renaming one for clarity:
- Keep `SkillDirectoryNotFound { skill_name }` as-is
- Rename `SkillsDirectoryNotFound { path }` to `SkillsPathNotFound { path }` or `SkillsDirectoryMissing { path }`

**Priority:** LOW (naming issue only, doesn't affect functionality)

---

## Recommendations Summary

### High Priority
None

### Medium Priority
1. **SkillNotFound** - Add actionable guidance and suggest searching/listing skills
2. **SkillsDirectoryNotFound** - Add context about whether this is normal and what to do
3. **RemoveFailed** - Add common causes and troubleshooting suggestions

### Low Priority
1. **NpxCommandFailed** - Consider adding network check suggestion
2. **MalformedSkillMetadata** - Consider adding frontmatter example
3. **NetworkUnavailable** - Add "check internet connection" suggestion
4. **ContainerInstallFailed** - Mention container-specific context
5. **MissingFrontmatter** - Add example frontmatter format
6. **Naming** - Consider renaming `SkillsDirectoryNotFound` to `SkillsPathNotFound` for clarity

---

## Conclusion

The SkillsError enum is well-documented with comprehensive comments for each variant. The Display implementations are generally clear and provide adequate information for debugging. The error messages strike a good balance between technical precision and user-friendliness.

**Key strengths:**
- Excellent documentation in comments
- Most errors provide relevant context
- Clear structure with fields for debugging
- Good variety covering all skills-related operations

**Areas for improvement:**
- Three error messages (SkillNotFound, SkillsDirectoryNotFound, RemoveFailed) would benefit from more actionable guidance
- Some technical messages could include user-friendly suggestions
- Consider minor naming improvements to reduce confusion

**Overall Grade:** B+ (Good with room for minor enhancements)

---

**Report completed:** 2026-02-20T02:26:53Z
