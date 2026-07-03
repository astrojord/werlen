# werlen

This [Ratatui] app generates stock lists for the magic item shops run by the not-so-humble retired adventurer Werlen in my D&D games. 
He stocks a variety of spell scrolls, low-rarity items, and a special stock of the good stuff for those that pay for his subscription service.

Users are responsible for providing their own shop stock, obtained with purchase(s) from WoTC or limited to the SRD, via the setting `stock_source`. The application expects a CSV file with columns `name`, `price`, `category`, `rarity` for items (leave empty for scrolls), and `level` for scrolls (leave empty for items).

The size of the three stock pools and the spell level/rarity range of items in the stock pools are configurable by the user at any point. Generation is slightly weighted towards lower level/rarity items.

[Ratatui]: https://ratatui.rs

### To do
- [ ] Improve UI elements - scrollbar widget on table border if rows are hidden, generation time indicator on top row, perhaps improved tab display
- [ ] Add file picker in settings tab to allow dynamic CSV stock source file name
