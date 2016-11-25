# anterofit

#### [retrofit](https://square.github.io/retrofit) | *verb*  \ˈre-trō-ˌfit, ˌre-trō-ˈfit\

* *to adapt an existing implement to suit a new purpose or need*

* *a Java library which generates REST API wrappers at runtime using class metadata* 

#### anterofit | *verb (neologism)* \ˈan-tə-(ˌ)rō-ˌfit\

* *to create an original implement to suit a particular purpose or need*

* *a Rust crate which generates type-safe REST API wrappers at compile time using macros*

Anterofit is a collection of Rust macros coupled to a lightweight, self-contained HTTP framework that
allows you to create strongly-typed wrappers around Rust APIs with ease.

Inspired by [Square's Retrofit](https://sqaure.github.io/retrofit), as referenced in the name, Anterofit is even
more strongly typed as everything that is feasible to check at compile-time, is. Runtime errors are,
with few exceptions, reserved for error conditions that can only be discovered at runtime.

---

