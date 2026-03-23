# Rosa Chess

Chess engine written in rust  
Made by rrakea  

Under active development  

## Docs

For docs run "cargo docs --open --workspace --document-private-items"

## Timeline

### Known Bugs

Forcing 3 Fold Repetitions in winning positions  
-> Mostly in endgames, but not exclusivly  
-> Mostly in endgames with big advantage but long range plans  
Pondering input crashes  
-> Pondering cli has not really been test  
Likes to move king early  

### Short Term

- Optimize TT entry size
- Make magic init run at comptime
- Make eval init run at comptime
- Put state & input handling in different files
- Optimize time checking to only happen every n nodes
- Add 50 move clock & 3 fold repetition
- Put position compare in test
- Better Draw checks in eval
- Delta pruning
- Check tt during quies
- Check if mv == mv is used somewhere, where mv.fuzzy_compare should be used

### Long Term

- Killer move heuristic
- Counter move heuristic
- Multithreading
