# alescript 🍺
*"A language that ferments ideas into reality."*  

alescript is **not just a language**; it’s an **experience**. It takes inspiration from the delicate **art of brewing**, where variables are **fermenting ingredients**, functions are **recipes**, and execution is a **carefully timed brewing process**.  

---

## Core concepts  

### Brews & ingredients (variables)  
Everything in alescript starts with a **brew**. these are the core units of computation.  

```alescript
brew lager from water, barley, hops, yeast.
```
This declares **lager**, which will start at **0% alcohol** and develop over time.  

The growth rate of **lager** is determined by the **ingredients**.  

- **water** is the base ingredient. it doesn't affect the alcohol content.  
- **barley** increases the growth rate by **1% per day**.  
- **hops** increase the growth rate by **0.5% per day**.  
- **yeast** increases the growth rate by **1.5% per day**.  

A brewmaster can specify how much of each ingredient to use.  

```alescript
brew lager from water, 1 barley, 2 hops, 1 yeast. // 2.5% growth per day
```

Brewing just **water** is not an error but it is meaningless.  

```alescript
brew badale from water. // 0% growth per day
wait for 5 days.
taste badale. // "0% ABV
```

---

### Fermentation (computation)  
Alescript doesn’t do **immediate** computations—it allows values to **ferment** over time. this forces programmers to think **strategically**, like a brewmaster.  

```alescript
wait for 5 days.
```
This will **increase all your beers' alcohol content** as time passes. computation is **not instant**—it brews.  

---

### Aging (precision timing)  
If you need **exact values**, you must **age your brew to perfection**.  

```alescript
age lager until is 5.2% abv.
```
Instead of manually incrementing numbers, you **let your brew mature**.  

Be aware that all beers share the **same timeline**. that means if you age one brew, all others will also **progress in time**.  

```alescript
brew stout from water, barley, hops, yeast.
brew porter from water, barley, hops.

age stout until is 6.0% abv.
taste porter. // "3% ABV"
```

---

### Tasting (printing output)  
When you're ready to see results, you can **taste** your brew.  

```alescript
taste lager.
```
This prints the **abv (alcohol by volume)** of lager.  

*example output:*  
*5.2% ABV*  

---

### Beer arithmetic  
alescript allows controlled **mathematical operations** using **brewing metaphors**.  

| operator | brewery metaphor | example usage |
|----------|-----------------|--------------|
| **addition (`+`)** | **mix** (blending two brews together) | `mix lager with stout.` |
| **multiplication (`*`)** | **double** (scaling up the brew) | `double porter by 3.` |
| **division (`/`)** | **dilute** (weakening a beer by adding water) | `dilute ipa by 2.` |

```alescript
brew ipa from water, barley, hops, yeast.
brew stout from water, barley, hops, yeast.

wait for 3 days.

mix ipa with stout.
double porter by 3.
dilute ipa by 2.
```

There is no **subtraction** in alescript. Seriously, how this is possible?

There are no negative values in alescript. If you want to **reduce** the alcohol content of a brew, you can **dilute** it with **water**.  

Note that **double** operation advances the global timeline. You need time to make your beer more potent.

---

### Recipes (function calls)  
Functions in alescript are **recipes**.  

```alescript
recipe my_pilsner() {
    brew pilsner from water, barley.
    age pilsner until 4.8% abv.
    pilsner
}
```

Last line of the function is the **return value**.  

```alescript
toast my_pilsner(). // "4.8% ABV"
```

Be aware that running `my_pilsner()` will advance the **global timeline**.

---


### Conditionals: judgment by the brewmaster  
Decisions in alescript are judgements, where a **brewmaster** decides if the brew is ready.  

```alescript
if lager is weaker than 4.0%: 
    toast "too weak!"
else:
    toast "perfectly brewed!"
```

In alescript, **boolean logic** is expressed through the **brewmaster's intuition**.  

```alescript
judge if lager is stronger than 5.0%:
    toast "a strong lager!"

judge if lager is not weaker than 5.0%:
    toast "a strong lager!"

judge if lager is weaker than 5.0%:
    toast "a weak lager!"
else:
    toast "a strong lager!"
```

It is important to understand that a brewmaster can only judge the **abv** of a brew basing on their intuition. Thus, the **comparison** is not **exact**. The imprecision of the judgement is always random but it is always within the range of **±10%** of the absolute value.
  

```alescript
judge if lager is stronger than 5.0%:
    toast "a strong lager!" // lager here can be between 4.5% and 100%

```

---

### Loops: brewing cycles  
Repetition in alescript follows **brewing cycles**.  

```alescript
repeat until lager is not less 5.0% abv:
    wait for 1 day.
```

---

### Kegging (stop growth)  

If the brew is **perfect**, you can **keg** it to **stop the growth**.  

```alescript
brew lager from water, barley, hops, yeast.

age lager until 5.0% abv.

keg lager.

wait for 3 days.

taste lager. // "5.0% ABV"

```


---

### Barrels (data structures)  
alescript supports **barrels** for storing multiple brews.  

```alescript
barrel taplist = [lager, stout, porter].

for each brew in taplist:
    taste brew.
```

A brewmaster can **add** or **remove** brews from a barrel.  

```alescript
barrel taplist = [].

add lager to taplist.
add stout to taplist.

remove lager from taplist.
```

A brewmaster can pick a brew from a barrel by **index**.  

```alescript
brew selected = taplist position 1.
```

Indexing starts at **1**.

---

## **Example: "hello, world!"**  
```alescript
toast "hello, world!".
```

---

## **Example: Fibonacci sequence**  
```alescript

recipe fibonacci(n) {
    brew a from water, barley.
    wait for 1 day.
    brew b from water, barley.
    brew temp from water.

    if n is 0:
        b
    else:
        repeat n times:
            relabel temp as a.
            mix a with b.
            relabel b as a.
            relabel a as temp.

        b
}

toast fibonacci(10). // "55% ABV"

```

---

## Why every programmer will want to migrate to alescript  
✅ **readable & expressive** – every line of alescript feels like **poetry**.  
✅ **time-oriented execution** – instead of **instant results**, computations "brew" over time.  
✅ **metaphor-driven** – every operation **tells a story**.  

---

This is **just the beginning**. alescript isn't **just a language**; it's a **philosophy of brewing code with patience and mastery**. 🍻  
