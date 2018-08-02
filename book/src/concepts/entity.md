# Entity

## What is an entity?

An `Entity` is simply a collection of `Component`s. At least conceptually. It represents a single object in your world.
For example, a car could be an entity, with its properties being `Component`s.

## Creating an entity

There are two common ways to create entities:
* From a `World` instance. See the relevant chapter in the book.
* From a `System` using `Entities`. See the system chapter in the book.

## Getting components of an entity

You can't! Well, at least not directly from an `Entity` instance.
As mentioned in the component book page, `Component`s are not directly attached to entities; they are inserted into storages.

`Storage` access and modification will be covered in the resource, world and system sections of the book.

