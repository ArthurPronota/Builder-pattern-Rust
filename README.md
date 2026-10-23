# Builder pattern в Rust

Твой `o_096` — правильная реализация typestate-builder: `.build()` не скомпилируется, пока не вызваны `.name()` и `.age()`. Разберём, что здесь хорошо, что можно улучшить, и уточним конспект.

## Что демонстрирует код

Три уровня typestate:

| Состояние | Доступный метод |
|-----------|-----------------|
| `Builder<NoName, NoAge>` | `new()`, `name()` |
| `Builder<N, NoAge>` | `age()` (при **любом** `N`) |
| `Builder<NameSet, AgeSet>` | `build()` |

Ключевая идея — **`impl<N> Builder<N, NoAge>`**: `age()` доступен независимо от того, установлено ли имя. Это позволяет вызывать `.name().age()` и `.age().name()` в любом порядке. Если бы было `impl Builder<NoName, NoAge>`, порядок стал бы жёстким.

## Что хорошо

1. **`impl<N> Builder<N, NoAge>`** — порядок вызовов свободный. Это правильное решение, а не `impl Builder<NoName, NoAge>`.
2. **`PhantomData<N>` и `PhantomData<A>`** — параметры считаются использованными, размер builder не растёт.
3. **`impl Into<String>`** в `name()` — принимает и `&str`, и `String`.
4. **`build()` только на `Builder<NameSet, AgeSet>`** — typestate работает.
5. **`unwrap()` в `build()` безопасен** — typestate гарантирует, что `Some` есть.

## Что можно улучшить

### 1. Проблема с `unwrap()` и `Default`

Если добавишь `#[derive(Default)]` или `Builder::default()`, можно получить `Builder<NoName, NoAge>` с `Some` внутри — инвариант сломается. Сейчас `new()` — единственный конструктор, это правильно. Но если кто-то добавит `Default`, typestate перестанет быть честным.

### 2. Дублирование полей в каждом методе

Каждый метод повторяет:

```rust
Builder {
    name: ...,
    age: self.age,
    _name_state: PhantomData,
    _age_state: PhantomData,
}
```

Это шумно. Помогает приватный хелпер:

```rust
impl<N, A> Builder<N, A> {
    fn transition<N2, A2>(self, name: Option<String>, age: Option<u32>) -> Builder<N2, A2> {
        Builder {
            name,
            age,
            _name_state: PhantomData,
            _age_state: PhantomData,
        }
    }
}
```

Тогда методы короче:

```rust
impl Builder<NoName, NoAge> {
    fn name(self, name: impl Into<String>) -> Builder<NameSet, NoAge> {
        self.transition(Some(name.into()), self.age)
    }
}
```

### 3. `unwrap()` → `expect` для диагностики

```rust
name: self.name.expect("typestate гарантирует Some"),
age: self.age.expect("typestate гарантирует Some"),
```

Если инвариант когда-нибудь сломается (например, кто-то добавит `Default`), `expect` даст понятное сообщение вместо «called `Option::unwrap()` on a `None` value».

### 4. `Drop` для незавершённого builder

Если builder брошен без `build()`, ничего страшного не происходит (нет ресурсов). Но если бы были — понадобился бы `Drop`. Сейчас не нужно.

### 5. `#[must_use]` на builder

```rust
#[must_use = "builder must be finished with .build()"]
struct Builder<N, A> { ... }
```

Компилятор предупредит, если builder создан и брошен без `build()`. Полезно, потому что typestate-цепочка без `build()` — почти наверняка ошибка.

### 6. Sealed-состояния

Сейчас любой может написать `Builder::<MyWeird, NoAge>` и вызвать `.age()`, потому что `impl<N> Builder<N, NoAge>` принимает **любой** `N`. Если хочешь запретить пользовательские состояния — сделай маркеры приватными или используй sealed-трейт:

```rust
mod state {
    pub trait NameState {}
    pub struct NoName;
    pub struct NameSet;
    impl NameState for NoName {}
    impl NameState for NameSet {}
}

struct Builder<N: state::NameState, A> { ... }
```

Тогда `Builder<MyWeird, NoAge>` не скомпилируется.

## Сравнение двух подходов из конспекта

Ты пишешь: «typestate жёстче, `Result` гибче». Это точное наблюдение.

### Typestate (твой вариант)

```rust
Builder::new().name("x").age(30).build()
```

- ❌ Пропуск поля — **ошибка компиляции**.
- ❌ Порядок не важен (благодаря `impl<N>`).
- ✅ Нет `unwrap()` в пользовательском коде.
- ❌ Сложнее API: типы состояний видны в документации.
- ❌ Плохо сочетается с условной логикой: `if cond { b = b.name(...) }` меняет тип — нужно аккуратно.

### `Result`-builder

```rust
Builder::new().name("x").build()?   // Err(MissingField("age"))
```

- ✅ Пропуск поля — **ошибка рантайма**, но обрабатываемая.
- ✅ Условная логика проще: тип builder не меняется.
- ✅ Проще API: один тип `Builder`.
- ❌ Ошибка ловится в рантайме, а не в компиляции.
- ❌ Нужны тесты на все комбинации пропусков.

### Когда что выбирать

| Ситуация | Рекомендация |
|----------|--------------|
| Обязательные поля **всегда** нужны | Typestate — ошибки ловятся компилятором |
| Поля условно обязательны | `Result`-builder — условная логика проще |
| Публичный API библиотеки | Typestate для критичных полей, `Result` для остальных |
| Много опциональных полей | `Default` + сеттеры (без typestate) |
| Быстрая итерация | `Result`-builder или `derive_builder` |

## Альтернатива: крейт `derive_builder`

Для типичных случаев не нужно писать всё вручную:

```rust
use derive_builder::Builder;

#[derive(Builder)]
struct User {
    name: String,
    age: u32,
}
// Генерирует UserBuilder с методами name(), age(), build() -> Result
```

Минус: `build()` возвращает `Result`, а не typestate. Но для многих проектов этого достаточно.

## Практические правила

1. **Typestate — когда обязательные поля действительно обязательны** и ошибка должна ловиться компилятором.
2. **`impl<N> Builder<N, NoAge>`** — для свободного порядка вызовов. Не привязывай `age()` к конкретному `N`, если порядок не важен.
3. **Приватные маркеры** — чтобы пользователь не подставил своё состояние.
4. **`#[must_use]`** на builder — предупредит о забытом `build()`.
5. **`expect` вместо `unwrap`** — понятная диагностика при нарушении инварианта.
6. **Хелпер `transition`** — убирает дублирование полей в каждом методе.
7. **`Result`-builder** — если нужна условная логика или гибкость.
8. **`derive_builder`** — для типичных случаев без typestate.

## Итог

Твой `o_096` — корректный typestate-builder. Главные улучшения:

- `impl<N>` для `age()` — уже сделано правильно.
- Приватные маркеры состояний — чтобы запретить `Builder<MyWeird, ...>`.
- `#[must_use]` — предупредит о забытом `build()`.
- `expect` вместо `unwrap` — лучше диагностика.
- Хелпер `transition` — меньше дублирования.

Typestate жёстче (`Result` гибче) — это верно. Выбор зависит от того, должны ли пропуски полей ловиться **компилятором** (typestate) или **рантаймом** (`Result`).
