# prohibit

A super simple tool to find and report "prohibited" elements in your project.

It applies the rules in a local `prohibited.json` file, an example one is
at `examples/cpp.json`.

This tool has been used has a pre-commit hook to stop committing code which
violates project-specific coding rules.  These might include not depending
on another directory, specific unallowed functions, etc.

Lines which contain the "overrule" text are ignored.

## Example

`examples/cpp.json` shows a simple example for a C++ project.  Lambdas are
not allowed, as is `iostream`, and unsized integer types.

The `inner` library is not allowed to access the `outer` library namespace.
It also cannot use `new` or `malloc`.  The project where this is used has
several usages of `new` which are allowed due to using the "overrule".

e.g.

```
T* value = new T;  // sample:override_prohibit
```

The `outer` library is prohibited from depending directly on `windows.h`.
