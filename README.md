# Lumber

![CI](https://github.com/foxfriends/lumber/workflows/Continuous%20Integration/badge.svg)

> Work in progress

Lumber is logic programming language, primarily intended for use embedded within Rust
applications as a scripting language. While other logic-based projects exist, I have
yet to find an easy way to describe and perform logical deduction from Rust programs.
Lumber attempts to solve that problem.

Goals:
1.  Easily interoperate with Rust: this is the whole reason the project was started.
2.  Simple implementation, ready for experimentation: the language is in early stages,
    so being able to try things quickly is important.

Non-goals:
1.  Replicate Prolog: I am hoping to take a fresher approach to Logic programming,
    not be a copy of Prolog which I find confusing at times.
2.  Incredible performance: I just want Lumber to work and be usable, performance
    of the runtime can come later.
