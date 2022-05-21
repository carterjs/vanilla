# Vanilla!

Vanilla is a simple programming language designed for text composition.

The syntax largely avoids parentheses, semicolons, and commas for readability. This trade off may not be worth it, so it's likely this will change to increase the clarity and capability of the language.

Every part of the language is immutable. Reassignments and shadowing are not permitted to aid in developer clarity. Since parentheses are not used for function calls, it would be confusing for function arity to change at any point.

## Functions

Functions read like sentences. In assignments, parameters follow the function name, separated by spaces. In calls, exactly the correct number of arguments will be consumed after the function identifier. There are no variadic functions, and functions cannot be overloaded.

```
add a b = a + b

print add 1 2 # prints 3
```

There are also "lambda" style functions that are useful when functions are being passed to other functions.

```
add = \ a b = a + b # lambdas usually aren't assigned like this, but the '=' may eventually change.

print add 1 2 # also prints 3
```

Some functions are built in. The most important built in functions are `print`, `println`, and `write`. More will appear later in this document.

## Control Flow

`if` and `else if` are available and they are also expressions. The type of each branch must be the same.

```
x = 5

if x < 5 (
    println "x < 5"
) else if x > 5 (
    println "x > 5"
) else (
    println "x == 5"
)

# Else/if used as an expression
println if x == 5 "x == 5" else "x != 5"
```

Notice that parentheses are used here.

## Types

Basic types for the language are the following:
- Number
- String
- Boolean
- Array
    - Every expression in the array must be the same type
- Block
- Function
- Nil
    - Note that `nil` refers to a type, not a value. `nil` is the absence of a value.

Everything is an expression and each expression must have a discernible type at compile time.

## Groups

Expressions can be grouped in 3 ways:
1. Groups
2. Arrays
3. Blocks

### Groups

Groups are the primary scoped entities of the language. They can be used to augment or clarify natural precedence, but they also concatenate their children, making them ideal for template-like 

```
x = 5 * (2 + 2) # x = 20
y = 5 * 2 + 2 # y = 12
```

### Arrays

Arrays allow us to preserve multiple individual expression values, but they will discard `nil` values.

```
things = [
    # This returns nil and will not be stored in the list
    x = 5

    x
    x + 1
    x + 2
] # things = [5 6 7]
```

Arrays can be indexed using a dot and indexing starts at 0.

```
print things.0 # prints 5
```

Arrays can be displayed or operated upon using the functional-style loops `loop` and `map`.

```
numbers = [1 2 3]
numbers_plus_one = map numbers \ n i = n + 1 # sets numbers_plus_one to [2 3 4]
```

```
loop numbers \ n i = println ("Number #" i + 1 " is " n) # prints each number
```

### Blocks

Blocks are groups that allow external access to their scoped local members.

In practice, they can be used like a key/value store. 

```
carter = {
    name = "Carter"
    age = 22
}

carter.name # "Carter"
carter.age + 1 # 23
```