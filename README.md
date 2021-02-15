# Lumber

![CI](https://github.com/foxfriends/lumber/workflows/Continuous%20Integration/badge.svg)
[![dependency status](https://deps.rs/repo/github/foxfriends/lumber/status.svg)](https://deps.rs/repo/github/foxfriends/lumber)

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

## Syntax

### `:-` -- predicate assignment, import, export

#### Assignment
```Lumber
greatedThan(A, B) :- A > B.

complexPredicate(A, B, C) :-
	A > B,
	other(B, C).
```

#### Import

???: Difference between `~::` and `^::`

```
:- use(~::game).
:- use(^::file(Statement/2)).
:- use(^::file(Statement/2, Seсond/4)).
:- use(@core).
:- use(@core::list).
```

#### Export

```Lumber
// /2 is count of parameters
:- pub(statement/2).
statement(A, B) :- A > B.
```


### `::-` -- ???

```Lumber
predicate(A, B) ::- other(A, B).
```


### `->>` -- ternary operator

```Lumber
If ->> Then ; Else.
HasRuins ->> Counter + 1 ; Counter.

// Invert
Statement ->> false; true.
```


### `=:=` -- equality
```Lumber
NewVar =:= OldVar + 1.
```


### `..` -- rest operator

```Lumber
statement(game { prop: NeededOne, .. }) :-

all([First, ..Rest]) :-
```


### `::` -- access operator

```Lumber
// import list from core
:- use(@core::list).

// ???
:- use(^::file)
```


### `//` -- comments

```Lumber
// one-line comment
```


### `Dot (.)` -- statement end

```Lumber
isEven(5).
```


### `Comma (,)` -- AND operator (statement delimiter)

```Lumber
complex(A, B) :-
	A < B,
	other(A, B).
```


### `Semicolon (;)` -- OR operator, ternary delimiter

#### OR

```Lumber
near(A, B) :-
	adjacent(A, B); A =:= B.
```

#### Ternary delimiter

```Lumber
If ->> Then ; Else.
```


### `Exclamation (!)` -- ???
```Lumber
statement(!Param, Param2) :-
```


### `Brackets ([])` -- Array

```Lumber
[_: Items(A, _, B)]
[Id : warrior(State, Id)]
```



