// Структура User
struct User {
    name:   String,
    age:    u32,
}

// реализация для User
impl User {
    fn name(&self) ->&str {
        &self.name
    }

    fn age(&self) ->u32 {
        self.age
    }

}

// Маркеры для состояния поля "name"
struct NoName ;     // имя не установлено
struct NameSet ;    // имя установлено

// Маркеры для состояния поля "age"
struct NoAge ;      // возраст не установлен
struct AgeSet ;     // возраст установлен

// построитель для установки name и age
struct Builder<N, A> {
    name:   Option<String>,
    age:    Option<u32>,
    _name_state:    std::marker::PhantomData<N>,
    _age_state:     std::marker::PhantomData<A>,
}

// создание нового bilder при не установленых name и age
impl Builder<NoName, NoAge> {
    fn new() ->Self {
        Self { 
            name: None, 
            age: None, 
            _name_state: std::marker::PhantomData, 
            _age_state: std::marker::PhantomData, 
        }
    }
}

// установка name при не установленом age
impl Builder<NoName, NoAge> {
    fn name(self, name: impl Into<String>) ->Builder<NameSet, NoAge> {
        Builder { 
            name: Some(name.into()), 
            age: self.age, 
            _name_state: std::marker::PhantomData, 
            _age_state: std::marker::PhantomData,
        }
    }
}

// Параметр N в этом impl-блоке — это обобщённый параметр типа 
// (generic type parameter), который обозначает текущее состояние 
// (любое состояние !!!) поля name в строителе.
// Установка age при любом name
impl<N> Builder<N, NoAge> {
    fn age(self, age: u32) ->Builder<N, AgeSet> {
        Builder { 
            name: self.name, 
            age: Some(age), 
            _name_state: std::marker::PhantomData, 
            _age_state: std::marker::PhantomData,
        }
    }
}

// Построитель User доступен только когда оба поля установлены
impl Builder<NameSet,AgeSet> {
    fn build(self) ->User {
        User { 
            name: self.name.unwrap(), 
            age: self.age.unwrap(),
        }
    }
}

fn main() {
    let builder = 
        Builder::new()
        .name("MyName")
        .age(30)
        .build() ;

    println!("Name: {}, Age: {}", builder.name(), builder.age()) ;
} /* Out:
Name: MyName, Age: 30
*/
