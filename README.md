# mdlog - markdown logbook/journaling made easier

## TODO
- Keep in touch with people: draw names from a csv file named ```people.csv``` (if it exists) and randomly adds them as call todos for generated templates 
```
# people.csv
# \n used as delimiter
Jacques Daniel
Johnathan Strider
...
```
```
# mdlog output
## Fri, 07.12.2018
- TODO: Call Jacques Daniel

## Sat, 08.12.2018
- TODO:  

## Sun, 09.12.2018
- TODO: Call Johnathan Strider
```
  - use the current timestamp as seed for the rng
  - 35% chance for a call; uniform probability for each person
- find TODOS: build simple parser and output generator to list todos and their associated tasks
```
# mdlog example file
...
- TODO: buy groceries
  - food
  - moar food
- TODO: commit stuff
- TODO: do other things
  - which have masssively long line lengths and because of that cant be added on the same line
- TODO: this has subtodos
  - TODO: figure out how to do it
  - needs to be lovely
...

```

```
# example output
# long output (>80 chars) should be broken into multiple lines; but short should stay in one line
# align to 2 digit numbers
 1. buy groceries; food; moar food  
 2. commit stuff
 3. do other things
    which have masssively long line lengths and because of that cant be added on the same line
 4. this has subtodos; needs to be lovely
    1. figure out how to do it
```
## License
[WFPL](https://www.wtfpl.net/)
