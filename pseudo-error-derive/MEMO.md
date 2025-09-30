## 仕様

`StackError`を容易に実装するための`derive`マクロ`StackError`を実装する

## 前提

- 利用者が別で`core::error::Error`を実装する


## 利用する`attribute`

- `#[source]`: `StackError`の次の宛先を示すマーカ
- `#[stack_error(end)]`: 次が`core::error::Error`しか実装されていないことを示すマーカ
- `#[location]`: `core::panic::Location`が格納されていることを示すマーカ


## 実装対象の構造体

`struct`,`tuple struct`,`struct variant`, `tuple variant`を対象にする

## 動作イメージ

`thiserror`に近いふるまいをイメージとする

1. フィールド名とついているアトリビュートを収集
3. `location`で用いるフィールドを特定する。これは次の優先順位で決定する
    1. `#[location]`アトリビュートが付いている
    2. フィールド名が`location`
    3. ない場合はエラー
2. `source`で用いるフィールドを特定する。これは次の優先順位で決定する
    1. `#[source]`アトリビュートが付いている
    2. `#[stack_error(end)]`アトリビュートが付いている
    3. フィールド名が`source`
4. 得た情報を使って`StackError`を実装する
    1. 2で`source`で用いるフィールドが特定できなかった場合は`None`を返す
    2. 特定されたフィールドに`#[stack_error(end)]`アトリビュートが付いている場合は`ErrorDetail::End`を返す
    3. それ以外の場合は`ErrorDetail::Stacked`を返す

### その他
- 複数フィールドの候補が見つかった場合、アトリビュートが付いているものを優先する。アトリビュートが複数存在する場合はエラーにする
- ジェネリクスが`source`に存在する場合`core::error::Error`をトレイト境界につける
- locationは`&'static core::panic::Location<'static>`であることを仮定する。これはユーザが守るルールにする
- enumの場合は各バリアントごとにlocationに該当するものが必須
- タプルライクの場合はアトリビュートが必須

### `struct`,`struct variant`の場合

```rust
use core::panic::Location;

#[derive(StackError)]
struct StructError{
    #[stack_error(end)]
    io: std::io::Error,
    location: &'static Location<'static>,
}

// 
#[derive(StackError)]
enum EnumStructError{
    E1{
        #[source]
        something: StructError,
        location: &'static Location<'static>,
    }
    E2 {
        source: StructError,
        #[location]
        error_location: &'static Location<'static>,
    },
    E3{
        // アトリビュートが優先される
        #[location]
        error_location: &'static Location<'static>,
        location: &'static Location<'static>,
    }
    // `#[source]`と`#[stack_error(end)]`、あるいは`#[location]`が複数ある場合はエラー
    // E4 {
    //     #[location]
    //     error_location: &'static Location<'static>,
    //     #[location]
    //     location: &'static Location<'static>,
    // }
    // Unit variantはエラー
    // E
}
```

### `tuple struct`, `tuple variant`の場合

```rust
use core::panic::Location;

#[derive(StackError)]
struct TupleError(#[source] std::io::Error, #[location] &'static Location<'static>);

#[derive(StackError)]
enum EnumTupleError{
    E1(#[source] TupleError, #[location] &'static Location<'static> )
}
```

## スコープ外

`core::error::Erorr`および`From`の自動実装
