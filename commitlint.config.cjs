// Commitlint configuration for conventional commit messages
// This configuration ensures commit messages follow conventional commit standards
module.exports = {
  // Extend the conventional commit configuration
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Define allowed commit types (build, chore, ci, docs, feat, fix, etc.)
    'type-enum': [
      2,
      'always',
      [
        'build',    // Changes that affect the build system or external dependencies
        'chore',    // Other changes that don't modify src or test files
        'ci',       // Changes to CI configuration files and scripts
        'docs',     // Documentation only changes
        'feat',     // A new feature
        'fix',      // A bug fix
        'perf',     // A code change that improves performance
        'refactor', // A code change that neither fixes a bug nor adds a feature
        'revert',   // Reverts a previous commit
        'style',    // Changes that do not affect the meaning of the code
        'test'      // Adding missing tests or correcting existing tests
      ]
    ],
    // Prevent certain cases in commit subject (no start-case, pascal-case, or upper-case)
    'subject-case': [2, 'never', ['start-case', 'pascal-case', 'upper-case']],
    // Commit subject cannot be empty
    'subject-empty': [2, 'never'],
    // Commit subject should not end with a period
    'subject-full-stop': [2, 'never', '.'],
    // Commit header should not exceed 72 characters
    'header-max-length': [2, 'always', 72]
  }
};
