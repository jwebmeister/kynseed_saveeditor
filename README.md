# kynseed_saveeditor

Savegame editor for the video game "Kynseed" by PixelCount Studios.

![Alt text](/../docs/docs/pics/kynseed_saveeditor_001.jpg?raw=true "kynseed_saveeditor")

## Version history & features
### v0.1
- Inventory table
    - Change quantity of each currently held items, per star rating (1-5).
    - Displays items unique id, name, type, and cost.
    - Change tables sort order via Inventory menu items.
- Inventory->"Give me 800 qty!" 
    - Sets quantity of all currently held items to 800, or it's max (e.g. 1), per star rating (1-5).
    - Sets quantity of all currently held partial cures (or has side effects) to zero (0).
- Inventory->"Give me 100 qty in larder"
    - Sets quantity of all items currently held in larders (e.g. owned shops, home cupboard) to 100 for their highest star rating.
- File->Save to overwrite the savegame with changes made.
    - Default is "Slot1_Autosave.xml".
    - Will also make a backup just prior to overwriting.
- File->Options to change editor settings
    - Can specify paths and filenames of required data and savegame files. Enable use of use modded game files, also see files within ".\saveedit_data\".
    - Default editor assumes it is run from Kynseed game directory (e.g. "...\steamapps\common\Kynseed\").
