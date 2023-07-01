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
- NOTE: The savegame editor needs access to folders and files from within your Kynseed game folder (specifically read to ".\Data" folder and files, read-write to ".\Saves" folder and files, and create-read-write to ".\saveedit_appconfig.toml") in order to function correctly.


## Version history & features
### v0.5.1
- Updated dependencies.

### v0.5
- "Save tree"
    - View and edit all data values within the savegame.
    - Add (duplicate node + children) or remove nodes.
    - Notes: 
        - *Use with caution*
        - No validation checks using the save tree, it's direct text editing.
        - You can mess up your savegame by inputting invalid values, adding or removing the wrong nodes. (See *.bak.# files for backups)
        - Editing values, adding or removing nodes in the save tree should also update the inventory table and player data window, and vice versa.
- Removed Inventory->"Inventory tree" (replaced with "Save tree")

### v0.4
- Inventory->"Inventory tree"
    - View and edit all data values under the inventory node of the savegame.

### v0.3
- "Player data"
    - Edit brass count (money)
    - Edit player characters stats
    - Edit tool level and xp

### v0.2
- Inventory table
    - Create a copy of an existing inventory item, via "+" button.
    - Editable UID of inventory items, i.e. can change the item type. 
        - Use with inventory item copy "+" button and "Loot reference" window to add new item types to your inventory.
    - Remove an existing inventory item, via "-" button.
    - Note: item quantities are only checked for valid values when you try changing the quantity. 
        - I suggest tweaking all the quantities if you change an items UID.
        - From my limited testing, excess (or invalid) quantity of items *shouldn't* break your savegame.
- Inventory->"Loot reference"
    - Opens a table listing all items and their UID, name, type, cost.
    - Filter table items by name, type, via editable text fields in header. 
        - Searches for items containing text, not case-sensitive. 
        - Leave blank for all items.
- Check for duplicate items (same UID) in the inventory upon pressing File->Save. 
    - Shows an error message with the specific duplicate UIDs, if there are any.
    - Prevents overwriting save until error is resolved.
- Updated the valid item quantity checks with latest game patch, e.g. illustrated book.

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
- File->"Save" to overwrite the savegame with changes made.
    - Default is "Slot1_Autosave.xml".
    - Will also make a backup just prior to overwriting.
- File->"Options" to change editor settings
    - Can specify paths and filenames of required data and savegame files. Enables use of modded game files, also see files within ".\saveedit_data\".
    - Default editor assumes it is run from Kynseed game directory (e.g. "...\steamapps\common\Kynseed\").
