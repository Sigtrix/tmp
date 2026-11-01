## Crafting Interpreters test suite

The upstream test runner requires Dart 2.19 (Dart 3 is incompatible).

Install the required Dart version using Homebrew:

```bash
brew install dart@2.19
```

Run the book's test suites as described [here](https://github.com/munificent/craftinginterpreters/tree/master#testing-your-implementation).

The first run requires an initial setup:
```bash
task book-setup BOOK_REPO=<path-to-book>
```
Then the implementation can be tested up to a given chapter (use absolute paths).
```bash
task test-book BOOK_REPO=<path-to-book> RLOX_DIR=<path-to-rlox-dir> CHAPTER=<book-chapter-name>
```
