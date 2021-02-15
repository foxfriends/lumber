# Lumber Guide

Lumber syntax and semantics are loosely based on those of Prolog, so if you have some familiarity
with that, much seen here may seem familiar.

Keep in mind that, as Lumber is very early-stages, this document may become outdated, or features
described here may not yet be implemented or may contain bugs and not work. If you encounter such
a case, do not hesitate to file an [issue](https://github.com/foxfriends/lumber/issues/new).

## Definitions

### Facts

A fact is the simplest form of definition, which describes something to be true.

```lumber
// Here are some animals
animal(cat).
animal(bear).
animal(fish).
animal(rabbit).

// And here are some plants
plant(carrot).
plant(lettuce).
plant(berry).

eats(cat, fish).      // Cats eat fish
eats(fish, fish).     // Fish eat other fish
eats(rabbit, carrot). // Rabbits eat carrots
eats(bear, fish).     // Bears eat fish
eats(bear, berry).    // Bears also eat berries
```

Given this program, you could query it like follows:

```
?- animal(cat).
    true.
?- animal(bird).
    false.
?- eats(cat, fish).
    true.
?- eats(cat, carrot).
    false.
?- animal(A).
    A = cat.
    A = bear.
    A = fish.
    A = rabbit.
?- eats(A, fish).
    A = cat.
    A = fish.
    A = bear.
```

Notice the use of capital letters in some of those queries: while identifiers starting with a
lower-case letter are values ("atoms"), identifiers starting with an upper-case letter are
variables.

### Rules

A rule allows computation of new truths based on other things the system knows about. They are
defined with a head (similar to a rule) and a body, separated by `:-`.

```lumber
// An herbivore is an animal that eats plants
herbivore(A) :- animal(A), plant(F), eats(A, F).
// A carnivore is an animal that eats animals
carnivore(A) :- animal(A), animal(F), eats(A, F).
// An omnivore is both an herbivore and a carnivore
omnivore(A) :- herbivore(A), carnivore(A).

// Rules can be recursive.
foodchain(A, F) :- eats(A, F).
foodchain(A, F) :- eats(A, B), foodchain(B, F).

?- herbivore(bear).
    false.
?- herbivore(rabbit).
    true.
?- carnivore(A).
    A = cat.
    A = bear.
    A = fish.
?- omnivore(A).
    A = bear.
```

Since a name can be reused many times, we refer to a predicate by its name and arity (number of
arguments), using notation like `carnivore/1` (predicate called `carnivore` with 1 argument) and
`foodchain/2` (predicate called `foodchain` with 2 arguments). Together, this is called the
predicate handle.

### Last-Rules

A last-rule works similarly to a regular rule, but unlike regular rules, once a last rule has
been matched, none of the following rules in the same definition will be considered. This can
help avoid some infinite loops. For example, try running `foodchain(fish, F)` as defined above
and you will find yourself in an infinite loop of `fish` eats `fish` eats `fish` eats `fish`...

This can (sort of) be alleviated by using a last-rule.

```lumber
foodchain(A, A) ::- eats(A, A).
foodchain(A, F) :- eats(A, F).
foodchain(A, F) :- eats(A, B), foodchain(B, F).
```

Now, `foodchain(fish, F)` will execute that first rule `foodchain(A, A) ::- eats(A, A).`, decide
that yes `fish` eats `fish`, but then not continue processing the following rules.

Unfortunately, `foodchain(bear, F)` is now broken, as it will first consider
`foodchain(bear, bear) ::- eats(bear, bear).`, decide that this is false, and then quit. A way
to make this more useful is [being considered](https://github.com/foxfriends/lumber/issues/56).

## Control-Flow

Within the body of a rule, predicates can be combined in a number of ways.

### Conjunction (and)

As seen above, the `,` control-flow operator is for conjunction. The query `p1, p2` will be
true only if both `p1` and `p2` are true. If either `p1` or `p2` is true in multiple ways,
the query `p1, p2` will be true in the cross-product of those possibilities.

For a more concrete example:

```lumber
a(1).
a(2).
b(3).
b(4).

?- a(A), b(B).
    A = 1, B = 3.
    A = 1, B = 4.
    A = 2, B = 3.
    A = 2, B = 4.
?- a(1), b(B).
    B = 3.
    B = 4.
?- a(A), b(2).
    false.
```

### Disjunction (or)

The `;` control-flow operator is for disjunction. The query `p1; p2` is true if either `p1`
or `p2` is true.

Using the same definitions for `a/1` and `b/1` as above:

```lumber
?- a(A); b(B).
    A = 1, B = _.
    A = 2, B = _.
    A = _, B = 3.
    A = _, B = 4.
```

Notice that only one side of the `;` is ever run at a time, leaving the other variable "unbound"
(`_`).

### Short-circuited conjunction (then)

The `->` control-flow operator performs short-circuited conjunction. In the query `p1 -> p2`, once
a solution for `p1` has been found, that solution is *fixed* and `p2` is executed as normal. If no
solution for `p1` could be found, the whole query is false.

Continuing with the example from above:

```lumber
?- a(A) -> b(B).
    A = 1, B = 3.
    A = 1, B = 4.
?- a(2) -> b(B).
    B = 3.
    B = 4.
?- a(A) -> b(3).
    A = 1.
?- a(3) -> b(B).
    false.
```

Notice how `A`, after being given a value on the left side of the `->`, is never given another
value. `B`, however, on the right side of the `->` is given multiple values as normal.

### Short-circuited disjunction (if)

The `->>` control-flow operator performs short-circuited disjunction, and is only usable alongside
the `;` operator (otherwise it would be the same as the single arrow `->`). The expression
`p1 ->> p2; p3` reads similarly to `if p1 then p2 else p3`.

Let's add a few more definitions to what we had above and try out a few more. It helps to compare
with using the single `->` as well:

```lumber
c(5).
c(6).

?- a(1) ->> b(B); c(B).
    B = 3.
    B = 4.
?- a(3) ->> b(B); c(B).
    B = 5.
    B = 6.
?- a(A) ->> b(B); c(B).
    A = 1, B = 3.
    A = 1, B = 4.
?- a(A) -> b(B); c(B).
    A = 1, B = 3.
    A = 1, B = 4.
    A = _, B = 5.
    A = _, B = 6.
```

### Unification

Sometimes it is useful to be able to perform an immediate unification of two values. This can be
done using the `=:=` unification operator.

```lumber
?- 1 =:= 1.
    true.
?- A =:= 1.
    A = 1.
?- 1 =:= 2.
    false.
```

## Values

There are a few different types of values in Lumber, which are similar to other languages:
1.  Integers (e.g. `1`, `2`, `0`, `10000`, `0xFF`, `0b11`): in Lumber, integers are unbounded.
    There is no `INT_MAX`.
2.  Rationals (e.g. `1.5`, `0.3`, `1.0000000000001`): similarly to integers, rationals are
    unbounded and of arbitrary precision. It will eventually be possible to construct rationals
    such as `1/3`, but for now this is not implemented.
3.  Strings (e.g. `"Hello world"`, `""`): these work as you might expect, but must be double
    quoted.
4.  Atoms (e.g. `hello`, `world`, `'Hello World'`, `#'It's me!'#`): these are similar to the
    "symbol" type found in languages such as Javascript and Ruby. Unquoted, simple atoms must
    start with a lowercase letter and only contain other letters, numbers, and underscores.
    Quoted atoms (single quotes) can contain any characters. If you want to include single
    quotes in your quoted atom, you can use `#` to build "stronger" quotes (e.g.
    `#'atom containing''#`, or `##'atom containing '#'##` and so on).
5.  Lists (e.g. `[1, 2, 3]`, `[a, b, c]`, `[[a, 1], [b, 2], [c, 3]]`): ordered lists, which can
    contain any types of values (including other lists).
6.  Records (e.g. `{ a: 1, b: 2 }`, `{ hello: world, goodbye: world }`): unordered key-value map,
    which can also contain any types of values (including other objects). Keys cannot be repeated,
    and each key can only contain one value (but that value can be a list).
7.  Structures (e.g. `valstruct(3)`, `liststruct [1, 2, 3]`, `recstruct { a: 1, b: 2 }`): A
    structure has a name and can contain one other value (in parentheses). If that value is a
    list or a record, the parentheses can be omitted.

Notably, there are no booleans. If you really need to manipulate booleans, the atoms `true` and
`false` are typically used.

## Patterns

At the core of Lumber is the idea of *unification*. Lumber programs work by executing queries
against the program. If an answer can be found by *unifying* the holes in the query with values
that still satisfy all the facts and rules of the program, then that answer is included in the
output.

Unification happens by writing patterns, which take on values at runtime, and unify as follows:
1.  Two values unify if they are exactly equal:
    1.  Integers, rationals, strings, and atoms are the same.
    2.  Lists contain the same values in the same order.
    3.  Records contain the same keys associated with the same values.
    4.  Structs have the same name and the same contents.
2.  A variable unifies with another value if the value contained by that variable unifies with
    the other value. If a variable is not yet bound, it will be bound to the other value which
    will be included in the output.
3.  A wildcard (`_`) works the same as a variable, but is not included in any outputs.
4.  The `?` pattern unifies only if the other pattern is not yet bound. For example, `?1 =:= _`
    will unify, but `?1 =:= 1` will not.
5.  The `!` pattern unifies only if the other pattern is bound. For example, `!1 =:= 1` will
    unify, but `!1 =:= _` will not.
6.  A list pattern may end with a "rest" pattern (e.g. `[X, ..Xs]`, `[1, 2, ..Rest]`), meaning
    the explicitly written patterns will unify with the prefix of the other list, and the end
    of the other list will unify with the rest pattern. If the actual value of the rest is not
    needed, the name can be omitted (e.g. `[X, .._]` and `[X, ..]` are equivalent).

7.  A record pattern may also end with a "rest" pattern (e.g. `{ a: b, ..C }`), which works
    similarly. The explicitly included keys will unify with those keys in the other record,
    and any not included will be unified with the rest. The name can be omitted from the rest
    pattern of records as well.

## Modules

For organizational purposes, it is useful to separate code into different modules. The Lumber module
system is inspired by that of Rust (though with a few flaws that still need to be addressed).

A module is declared with `mod` directive (a directive is code that directs the interpreter how to
interpret, rather than code that exist in the program). The `mod` directive in use looks like this:

```lumber
:- mod(my_module).
```

When such a module declaration is encountered in the code, Lumber will look for the a file with the
same name as the module or for a directory by that name with a `mod` file in it.

For example, we can split up those definitions from before into some modules like this:

```lumber
/** ./main.lumber **/
:- mod(animals).
:- mod(plants).
:- mod(foodchain).


/** ./plants.lumber **/
plant(carrot).
plant(lettuce).
plant(berry).


/** ./animals.lumber **/
animal(cat).
animal(bear).
animal(fish).
animal(rabbit).


/** ./foodchain/mod.lumber **/
:- mod(eats).
:- mod(vores).

foodchain(A, F) :- eats(A, F).
foodchain(A, F) :- eats(A, B), foodchain(B, F).


/** ./foodchain/eats.lumber **/
eats(cat, fish).      // Cats eat fish
eats(fish, fish).     // Fish eat other fish
eats(rabbit, carrot). // Rabbits eat carrots
eats(bear, fish).     // Bears eat fish
eats(bear, berry).    // Bears also eat berries


/** ./foodchain/vores.lumber **/
herbivore(A) :- animal(A), plant(F), eats(A, F).
carnivore(A) :- animal(A), animal(F), eats(A, F).
omnivore(A) :- herbivore(A), carnivore(A).
```

### Exporting

Definitions can be exported from modules. For code outside of the module, only those exported
definitions can be accessed. This is done using the `pub` directive, which takes a single
handle of a predicate accessible in this module.

```lumber
:- pub(predicate/3).
```

Using this, we can add exports to the modules above making all of the predicates accessible:

```lumber
/** ./main.lumber **/
:- mod(animals).
:- mod(plants).
:- mod(foodchain).


/** ./plants.lumber **/
:- pub(plant/1).

plant(carrot).
plant(lettuce).
plant(berry).


/** ./animals.lumber **/
:- pub(animal/1).

animal(cat).
animal(bear).
animal(fish).
animal(rabbit).


/** ./foodchain/mod.lumber **/
:- mod(eats).
:- mod(vores).

:- pub(foodchain/2).

foodchain(A, F) :- eats(A, F).
foodchain(A, F) :- eats(A, B), foodchain(B, F).


/** ./foodchain/eats.lumber **/
:- pub(eats/2).

eats(cat, fish).      // Cats eat fish
eats(fish, fish).     // Fish eat other fish
eats(rabbit, carrot). // Rabbits eat carrots
eats(bear, fish).     // Bears eat fish
eats(bear, berry).    // Bears also eat berries


/** ./foodchain/vores.lumber **/
:- pub(herbivore/1).
:- pub(carnivore/1).
:- pub(omnivore/1).

herbivore(A) :- animal(A), plant(F), eats(A, F).
carnivore(A) :- animal(A), animal(F), eats(A, F).
omnivore(A) :- herbivore(A), carnivore(A).
```

### Importing

Once a predicate is exported from a module, other modules are then able to import that predicate
using the `use` directive. This directive is a bit more complex than the other ones, as it takes
a *module path*, and then (optionally) a list of predicates.

Let's just add some imports to the modules above, as they are best described by example.

```lumber
/** ./main.lumber **/
:- mod(animals).
:- mod(plants).
:- mod(foodchain).

:- use(animals(animal/1)). // imports animal/1 from module `animals` (relative to the current module)
:- use(plants(plant/1)).
:- use(foodchain). // imports all predicates that are available in module foodchain
:- use(foodchain::eats(eats/2)). // import eats/2 from the module `eats`, found in module `foodchain`.

/** ./plants.lumber **/
:- pub(plant/1).

plant(carrot).
plant(lettuce).
plant(berry).


/** ./animals.lumber **/
:- pub(animal/1).

animal(cat).
animal(bear).
animal(fish).
animal(rabbit).


/** ./foodchain/mod.lumber **/
:- mod(eats).
:- mod(vores).

:- use(eats(eats/2)).
:- use(vores). // Imports all public predicates from module `vores`

// Let's re-export those 3 predicates from module `vores` also
:- pub(herbivore/1).
:- pub(carnivore/1).
:- pub(omnivore/1).

:- pub(foodchain/2).

foodchain(A, F) :- eats(A, F).
foodchain(A, F) :- eats(A, B), foodchain(B, F).


/** ./foodchain/eats.lumber **/
:- pub(eats/2).

eats(cat, fish).      // Cats eat fish
eats(fish, fish).     // Fish eat other fish
eats(rabbit, carrot). // Rabbits eat carrots
eats(bear, fish).     // Bears eat fish
eats(bear, berry).    // Bears also eat berries


/** ./foodchain/vores.lumber **/
:- pub(herbivore/1).
:- pub(carnivore/1).
:- pub(omnivore/1).

// ^:: refers to the parent module (foodchain), similar to how ../ refers to the parent directory
// in a file path.
:- use(^::eats(eats/2)). // import eats/2 from the `eats` module found in the parent

// ~:: refers to the root module (main.lumber), similar to how `~/` refers to the home directory
// in a file path.
:- use(~::animals(animal/2)). // import animal/2 from the `animals` module found in the root
:- use(~::plants(plant/2)).   // import plant/2 from the `plants` module found in the root

herbivore(A) :- animal(A), plant(F), eats(A, F).
carnivore(A) :- animal(A), animal(F), eats(A, F).
omnivore(A) :- herbivore(A), carnivore(A).
```

### Libraries

Lumber also supports libraries - external modules that can be shared between many Lumber programs.
As of now, the only available library is the Lumber standard library `core`, but you could write
your own libraries if you felt like it. Predicates from libraries are imported much like other
predicates from regular modules, but the library's name is prefixed with `@`. For example, to
import the standard `print/1` predicate from `core` would be:

```lumber
:- use(@core(print/1)).
```

### Use without importing

If importing the predicate is undesirable (e.g. because you have defined another predicate with
the same name and arity in the current module), you can reference that external predicate without
importing it just by writing the whole module path in the calling code.

For example instead of importing anything in the module `vores` above, we could just reference
all those predicates directly:

```lumber
herbivore(A) :- ~::animal::animal(A), ~::plant::plant(F), ^::eats::eats(A, F).
carnivore(A) :- ~::animal::animal(A), ~::animal::animal(F), ^::eats::eats(A, F).
omnivore(A) :- herbivore(A), carnivore(A).

// Now since eats/2 is not taken, we can define it:
eats(A, meat) :- carnivore(A).
eats(A, plants) :- herbivore(A).
```

### Aliasing

Another alternative to consider when you may encounter predicate name conflicts is aliasing.
In the `use` directive, the special form `alias(_, as: _)` can be used to rename a predicate:

```lumber
:- use(~::animals(animal/1)).
:- use(~::plants(plant/1)).
:- use(^::eats(alias(eats/2, as: eatsOther/2))).

herbivore(A) :- animal(A), plant(F), eatsOther(A, F).
carnivore(A) :- animal(A), animal(F), eatsOther(A, F).
omnivore(A) :- herbivore(A), carnivore(A).

// eats/2 is again not taken, so we can define it:
eats(A, meat) :- carnivore(A).
eats(A, plants) :- herbivore(A).
```

## Operators and Expressions

>   Operators and expressions are a relatively new feature, and are somewhat complicated...
>   Beware of bugs!

Operators and expressions in Lumber are a bit different from operators and expressions that you
may have experience with in other languages, so using them requires a bit of care.

Operators can be defined using the `op` directive, which takes:
1.  an operator symbol;
2.  the handle of the predicate that the operator is implemented by; and, optionally,
3.  an associativity (`left` or `right`) and precedence (integer from 1 to 9).

For example, the `+` operator as is commonly known is defined in the standard library as follows:

```lumber
:- op(+, add/3, left, 6).
```

Operators are imported and exported from modules just like predicates:

```lumber
:- pub(+).

// Then, to import
:- use(@core(+)).
```

There are 4 locations where operators can be used:
1.  Infix expression operators (e.g. `X =:= A + B`): 3-arity + associativity + precedence
2.  Prefix expression operators (e.g. `X =:= -A`): 2-arity + associativity + precedence
3.  Infix predicate operators (e.g. `A < B`): 2-arity, no associativity or precedence
4.  Prefix predicate operators (e.g. `~X`): 1-arity, no associativity or precedence

As their names would imply, predicate operators can be used in place of predicates in a definition,
for example, the following are equivalent (given `:- op(<, lt/2)`)

```lumber
:- use(@core(lt/2)).
lessThan2(A) :- lt(A, 2).

:- use(@core(<)).
lessThan2(A) :- A < 2.
```

Expression operators can be used anywhere an expression can go (which is anywhere a pattern can go
in the body of a definition). They cannot be used in the heads of predicates. Given the `op`
definition for `+` as above, the following are equivalent:

```lumber
:- use(@core(add/3)).
sum(A, B, C, Sum) :-
    add(A, B, Sum1),
    add(Sum1, C, SUm).

:- use(@core(+)).
sum(A, B, C, D) :-
    D =:= A + B + C.

// due to associativity:
:- use(@core(+)).
sum(A, B, C, D) :-
    D =:= ((A + B) + C).
```

Notice that the left and right hand side of the operator are the first two arguments to the
underlying predicate, and the third argument is the output.

## Tests

Lumber provides a basic unit-testing feature via the `test` directive. This directive takes a
single query, and when Lumber is run in test-mode, all tests will be run. A test passes if the
query has any answers, and fails if not.

```lumber
predicate(a, b).
predicate(c, d).

:- test(predicate(a, b)). // pass
:- test(predicate(a, _)). // pass
:- test(predicate(a, c)). // fail
:- test(predicate(a, c) ->> false; true). // pass
```

When Lumber is run normally, all tests are ignored and omitted from the resulting program.
