# Specifikace

## Úvod a motivace

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

Druhý z nich spočívá v definici schéma.
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
Pokud by tedy oba popisy schéma nebyly zcela identické,
tak by se při deserializaci data pomíchala způsobem, který by se velmi obtížně odlaďoval.

Třetí problém souvisí s druhým problémem a spočívá ve verzování schéma.
Často se stane, že se v nové verzi aplikace struktura přenášených dat změní.
Správce aplikace typicky do produkce nasadí novou verzi serveru i klientů ve stejný okamžik,
někteří uživatelé mohou mít webovou stránku během aktualizace načtenou,
a stránku obnovit je pravděpodobně okamžitě nenapadne.
Pak nastane situace, kdy se spolu snaží bavit nová verze serveru a stará verze klienta.
Symptomy budou podobné, jako když se programátor splete v přepisu schéma z Rustu do TypeScriptu.
Část aplikace pak pravděpodobně přestane fungovat, a to těžko předvídatelným způsobem.
V lepším případě se jen někde na stránce zobrazí `undefined` nebo `[object Object]`.
Tento problém je ještě důležitější v mobilních aplikacích,
kde uživatelé mohou dále používat starou verzi aplikace týdny po vydání nové verze.
Obdobně jako u předchozího problému se tento problém zhorší, pokud aplikace přejde na binární formát.

Tento projekt si klade za cíl vyřešit druhý a třetí ze zmiňovaných problémů,
a to pomocí univerzálního jazyka na popis schéma a nástrojů, které umožní spravovat jeho verzování.
Snaha ale je ponechat volnou cestu k vyřešení i prvního problému,
a to pomocí budoucího návrhu kompaktního datového formátu umožňující posílat změny.

## Popis projektu

V rámci přípravy projektu byl navržen níže popsaný jazyk pro zápis *schéma*,
množiny algebraických datových typů myšlených pro ukládání a přenos informací
po jejich serializaci do nějakého datového formátu.
Cílem projektu je implementace *překladače*, programu, který zápis schéma v tomto jazyce přeloží do typů
v běžném programovacím jazyce, v rámci tohoto projektu jen do jazyka Rust.

Druhou funkcí překladače bude pomoc se správou verzí schéma vytvářením migrací.
Vygenerované datové typy umístí do jmenného prostoru pojmenovaného podle verze schéma.
V případě, že vznikne nová verze schéma, umožní překladač do programu přidat typy pro tuto verzi,
zatímco budou typy pro starou verzi zachovány.
Dále automaticky vygeneruje *migrační funkce* pro všechny typy, které se od předchozí verze schéma nezměnili.
Tyto funkce převedou typ z předchozí verze schéma na typ v nové verzi schéma, nebo naopak.
Pro typy, které se změnili, vygeneruje signatury migračních funkcí.
Programátorovi bude tedy stačit doplnit těla této hrstky funkcí.

Nakonec překladač k vygenerovaným typům přidá atributy,
které umožní typy serializovat a deserializovat pomocí knihovny `serde` do řady různých formátů.
