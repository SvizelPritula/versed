# Specifikace

## Popis projektu

V rámci přípravy projektu byl navržen níže popsaný jazyk pro zápis *schéma*,
množiny algebraických datových typů myšlených pro ukládání a přenos informací
po jejich serializaci do nějakého datového formátu.
Cílem projektu je implementace *překladače*, programu, který zápis schéma v tomto jazyce přeloží do typů
v běžném programovacím jazyce, v rámci tohoto projektu jen do jazyka Rust.

Druhou funkcí překladače bude pomoc se správou verzí schéma vytvářením migrací.
Vygenerované datové typy umístí do jmeného prostoru pojmenovaného podle verze schéma.
V případě, že vznikne nová verze schéma, umožní překladač do programu přidat typy pro tuto verzi,
zatímco budou typy pro starou verzi zachovány.
Dále automaticky vygeneruje *migrační funkce* pro všechny typy, které se od předchozí verze schéma nezměnili.
Tyto funkce převedou typ z předchozí verze schéma na typ v nové verzi schéma, nebo naopak.
Pro typy, které se změnili, vygeneruje signatury migračních funkcí.
Programátorovi bude tedy stačit doplnit těla této hrstky funkcí.

Nakonec překladač k vygenerovaným typům přidá atributy,
které umožní typy serializovat a deserializovat pomocí knihovny `serde` do řady různých formátů.
