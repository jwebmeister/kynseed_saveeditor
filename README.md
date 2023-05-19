# kynseed_saveeditor

Savegame editor for the video game "Kynseed" by PixelCount Studios.

![Alt text](./docs/kynseed_saveeditor_001.jpg?raw=true "kynseed_saveeditor")

## Install instructions
- Download latest release from https://github.com/jwebmeister/kynseed_saveeditor/releases/latest
- Copy the saveeditor files and folders to within your Kynseed game folder:
    - unzip from "kynseed_saveeditor*.zip" (into game folder):
        - "kynseed_saveeditor.exe"
        - "saveedit_data" folder and containing files
    - e.g. "\steamapps\common\Kynseed\kynseed_saveeditor.exe"
- Run "kynseed_saveeditor.exe" from within the Kynseed game folder.  
    - It will prompt in the bottom status bar if there are any errors.  
    - You can make changes to the editors settings within the File->Options menu, or change the .toml file generated after first run of the editor.
- NOTE: The savegame editor needs access to folders and files from within your Kynseed game folder (specifically read to ".\Data", read-write to ".\Saves", folders and files) in order to function correctly.


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
