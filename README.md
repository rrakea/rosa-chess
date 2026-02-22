# Rosa Chess

Chess engine written in rust  
Made by rrakea  

Under active development  

## Docs

For docs run "cargo docs --open --workspace --document-private-items"

## Timeline

### Known Bugs

- Forcing 3 Fold Repetitions in winning positions

### Short Term

- Promotion mv gen separate + mv ordering?
- Optimize TT entry size
- Make magic init run at comptime
- Make eval init run at comptime
- Separate check_for_legality in make
- Put state & input handling in different files
- Optimize time checking to only happen every n nodes
- Add 50 move clock & 3 fold repetition
- Put position compare in test
- Better Draw checks in eval
- Delta pruning
- Check tt during quies

### Long Term

- Killer move heuristic
- Counter move heuristic
- Multithreading
