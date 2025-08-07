# Specifikace

Tento dokument obsahuje specifikaci ročníkového projektu _Popis a migrace schématu DTO_,
který v létě 2025 zpracovává Benjamin Swart.

## Základní informace o projektu

### Úvod a motivace

Ve svém volném čase občas programuji multiplayerové hry pro volnočasové akce, které mají podobu webové aplikace.
Tyto aplikace mají frontend napsaný v TypeScriptu a backend napsaný v Rustu.
Zajímavé jsou tím, že často potřebují synchronizovat stav hry ze serveru na klienty v reálném čase.
Tento stav se většinou posílá zakódovaný v JSON pomocí WebSockets.
Při vývoji těchto aplikací jsem narazil na tři problémy spojené s výměnou informací.

První z nich spočívá v tom, že jsou přenášené objekty poměrně velké.
V dnešní době je sice možné po internetu poslat opravdu hodně dat,
ale větší objem dat může zvýšit latenci hry a lidem hrajících hru mimo dosah WiFi může plýtvat mobilní data.
Částečným řešením tohoto problému by bylo místo JSON používat nějaký kompaktní, binární formát.
Dále by pomohlo nepřenášet po každé změně celý stav hry, ale jen ty části, které se změnili.

Druhý z nich spočívá v definici schématu.
Není to problém specifický pro hry, ale týká se všech aplikací, co komunikují po síti.
Rust i TypeScript jsou silně typované jazyky, takže je potřeba (nebo alespoň silně žádoucí)
definovat datový typ pro každý přenášený objekt.
To je potřeba udělat pro každý z obou jazyků zvlášť, což zabere nějaký čas.
Pokud TypeScriptové typy neodpovídají těm v Rustu, tak deserializovaná data nebudou mít tvar,
který klientský kód očekává, a aplikace se pravděpodobně rozbije.
Tyto chyby naštěstí většinou není těžké při testování zachytit a opravit,
ale stále jejich opravou programátor nějaký čas stráví.
Kdybych navíc přešel na nějaký binární formát, který neobsahuje popis své vlastní struktury,
jak jsem zmiňoval u prvního problému, tak by se problém ještě zhoršil.
Ke správné deserializaci takového formátu je potřeba předem přesně znát použité schéma.
Pokud by tedy oba popisy schématu nebyly zcela identické,
tak by se při deserializaci data pomíchala způsobem, který by se velmi obtížně odlaďoval.

Třetí problém souvisí s druhým problémem a spočívá ve verzování schématu.
Často se stane, že se v nové verzi aplikace struktura přenášených dat změní.
Správce aplikace typicky do produkce nasadí novou verzi aplikace pro server i klienty ve stejný okamžik.
Někteří uživatelé mohou mít webovou stránku během aktualizace načtenou,
a stránku obnovit je pravděpodobně okamžitě nenapadne.
Pak nastane situace, kdy se spolu snaží bavit nová verze serveru a stará verze klienta.
Symptomy budou podobné, jako když se programátor splete v přepisu schématu z Rustu do TypeScriptu.
Část aplikace pak pravděpodobně přestane fungovat, a to těžko předvídatelným způsobem.
V lepším případě se jen někde na stránce zobrazí `undefined` nebo `[object Object]`.
Tento problém je ještě důležitější v mobilních aplikacích,
kde uživatelé mohou dále používat starou verzi aplikace týdny po vydání nové verze.
Obdobně jako u předchozího problému se tento problém zhorší, pokud aplikace přejde na binární formát.

Tento projekt si klade za cíl vyřešit druhý a třetí ze zmiňovaných problémů,
a to pomocí univerzálního jazyka na popis schématu a nástrojů, které umožní spravovat jeho verzování.
Snaha ale je ponechat volnou cestu k vyřešení i prvního problému,
a to pomocí budoucího návrhu kompaktního datového formátu umožňující posílat změny.

### Popis projektu

V rámci projektu bude navržen jazyk pro zápis *schématu*,
množiny algebraických datových typů myšlených pro ukládání a přenos informací
po jejich serializaci do nějakého datového formátu.
Cílem projektu je implementace *překladače*, programu, který zápis schématu v tomto jazyce přeloží do typů
v běžném programovacím jazyce, v rámci tohoto projektu jen do jazyka Rust a TypeScript.

Druhou funkcí překladače bude pomoc se správou verzí schématu vytvářením migrací.
Vygenerované datové typy umístí do jmenného prostoru pojmenovaného podle verze schématu.
V případě, že vznikne nová verze schématu, umožní překladač do programu přidat další typy pro tuto verzi,
zatímco typy pro starou verzi budou zachovány.
Dále automaticky vygeneruje *migrační funkce* pro všechny typy, které se od předchozí verze schématu nezměnili.
Tyto funkce převedou typ z předchozí verze schématu na typ v nové verzi schématu, nebo naopak.
Pro typy, které se změnili, vygeneruje signatury migračních funkcí.
Programátorovi bude tedy stačit doplnit těla této hrstky funkcí.
V rámci projektu budou migrace podporované jen v jazyce Rust.

Nakonec překladač k vygenerovaným typům v Rustu přidá atributy,
které umožní typy serializovat a deserializovat pomocí knihovny Serde do řady různých formátů,
mimo jiné JSON kompatibilní s vygenerovanými typy v TypeScriptu.

### Hlavní funkce

Program bude konzolová aplikace s dvěma hlavními funkcemi:

- generace typů v cílových jazycích na základě schématu, a
- generace triviálních částí migračních funkcí na základě dvojice schémat.

### Motivační příklad užití

Vývojář chce naprogramovat část aplikace, ve které budou moci uživatelé spravovat svůj profil.
Napíše následující popis schématu:
_(Přesná syntaxe jazyka není finální.)_

```
version v1;

User = struct {
    name: string,
    contact: Contact,
};

Contact = enum {
    phone: number,
    email: string,
};
```

Backend aplikace bude posílat objekty typu `User` frontendu, ten je bude upravené posílat zpátky.
Program vygeneruje následující typy:
_(Podoba opět není finální.)_

```rust
pub mod v1 {
    #[derive(Clone, Serialize, Deserialize)]
    pub struct User {
        pub name: String,
        pub contact: Contact,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub enum Contact {
        Phone(i64),
        Email(String),
    }
}
```

Obdobné typy nechá vývojář překladač vygenerovat i pro TypeScript.
Poté napíše backend, který objekt `User` vytvoří a pomocí knihovny Serde serializuje do JSON.
Frontend si JSON stáhne pomocí HTTP požadavku, který bude obsahovat (např. v URL nebo v hlavičce) verzi `v1`.
Poté ho deserializuje ho pomocí `JSON.parse(...)` a přetypuje výsledek na typ `User`.
Uživatelem upravenou verzi objektu frontend obdobně serializuje a odešle na backendový server
požadavkem obsahující verzi.

Později, po vydání aplikace, vývojář schéma upraví:

```diff
- version v1;
+ version v2;
  
  User = struct {
      name: string,
+     age: enum { age: number, unknown },
-     contact: Contact,
+     contact: [Contact],
  };
  
  Contact = enum {
      phone: number,
      email: string,
  };
```

Překladač vygeneruje nové typy, které vývojář do projektu přidá:

```rust
pub mod v2 {
    pub struct User {
        pub name: String,
        pub age: UserAge,
        pub contact: Vec<Contact>,
    }

    pub enum UserAge {
        Age(i64),
        Unknown,
    }

    pub enum Contact {
        Phone(i64),
        Email(String),
    }
}
```

Dále vygeneruje migrační funkce.
Pro typ `Contact` je vygeneruje automaticky, protože se nezměnil,
zatímco do funkcí pro typ `User` bude potřeba nějaký kód doplnit:
_(Podoba opět není finální.)_

```rust
fn upgrade_user(user: v1::User) -> v2::User {
    let v1::User { name, contact } = user;

    v2::User {
        name,
        age: todo!(),
        contact: todo!(),
    }
}

fn downgrade_user(user: v2::User) -> v1::User {
    let v2::User { name, age, contact } = user;

    v1::User {
        name,
        contact: todo!(),
    }
}

fn upgrade_contact(contact: v1::Contact) -> v2::Contact {
    match contact {
        v1::Contact::Phone(phone) => v2::Contact::Phone(phone),
        v1::Contact::Email(email) => v2::Contact::Email(email),
    }
}

fn downgrade_contact(contact: v2::Contact) -> v1::Contact {
    match contact {
        v2::Contact::Phone(phone) => v1::Contact::Phone(phone),
        v2::Contact::Email(email) => v1::Contact::Email(email),
    }
}
```

Programátor pak funkce `upgrade_user` a `downgrade_user` doplní třeba takto:

```rust
fn upgrade_user(user: v1::User) -> v2::User {
    let v1::User { name, contact } = user;

    v2::User {
        name,
        age: v2::UserAge::Unknown,
        contact: vec![upgrade_contact(contact)],
    }
}

fn downgrade_user(user: v2::User) -> v1::User {
    let v2::User { name, age, contact } = user;

    v1::User {
        name,
        contact: contact[0],
    }
}
```

Vývojář vygeneruje nové TypeScriptové typy pro frontend a staré TypeScriptové typy zahodí.
Frontend i backend upraví, aby pracoval s novými typy.
Ve frontendu dále změní verzi uvedenou v požadavcích na `v2`,
zatímco do backendu přidá kus kódu, který verzi v požadavku zkontroluje.
Pokud požadavek udává verzi `v1`, tak server nahraná data převede do verze `v2` pomocí funkce `upgrade_user`
a odesílaná data převede do `v1` pomocí `downgrade_user`.
Díky tomu budou moci staré verze frontendu fungovat s novým serverem.

## Technická stránka projektu

### Použité technologie

Projekt bude implementován v jazyce Rust.
Bude podporovat generování kódu pro následující jazyky:

- Rust
- TypeScript

V jazyce Rust je k serializaci zamýšleno užití knihovny [Serde](https://serde.rs/),
v jazyce TypeScript je zamýšleno užití standardní knihovny, konkrétně objektu `JSON`.

### Prostředí aplikace

Program bude odladěn pro instrukční sadu x86-64 a operační systém Linux,
tedy cíl `x86_64-unknown-linux-gnu` překladače Rustu.
Nic by ale nemělo bránit tomu, aby program šel sestavit a spustit na jakékoliv platformě
s [podporou Rustu](https://doc.rust-lang.org/stable/rustc/platform-support.html)
úrovně _Tier 1 with Host Tools_ nebo _Tier 2 with Host Tools_.

## Uživatelské rozhraní programu

Překladač bude implementován jako konzolová aplikace s několika podpříkazy.
Každý příkaz odpovídá jedné funkci projektu a je tedy popsán v popisu příslušné funkce.
Obecně každý příkaz jako argumenty dostane
cestu k jednomu nebo více souborům obsahujících popis schématu,
název cílového jazyka a cestu ke složce uvnitř projektu, kam má uložit vygenerované soubory.
Jakékoliv potkané chyby, zejména chyby v syntaxi nebo sémantice schématu,
vypíše program na standardní chybový výstup.
