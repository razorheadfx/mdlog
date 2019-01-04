# mdlog - markdown logbook/journaling made easier

## TODO
- Keep in touch with people: draw names from a csv file named ```people.csv``` (if it exists) and randomly adds them as call todos for generated templates 
```
# people.csv
# \n used as delimiter
Simon
George Washington
...
```
```
# mdlog output
## Friday, 14.09.2018
- TODO: Call Simon
```
  - use the current timestamp as seed for the rng
  - 35% chance for a call; uniform probability for each person

## License
[WFPL](https://www.wtfpl.net/)
