meta:
  id: mbe
  file-extension: mbe
  endian: le
  encoding: UTF-8

seq:
  - id: magic
    contents: "EXPA"
  - id: number_of_entries
    type: u4
  - id: sheets_headers
    repeat: expr
    repeat-expr: number_of_entries
    type: sheet_header(_index == 0)
  - id: chunk
    type: chnk

types:
  sheet_header:
    params:
      - id: is_first_header
        type: bool
    seq:
      - id: pad
        contents: [0,0,0,0]
        if: is_first_header == false
      - id: length_of_entry_name
        type: u4
      - id: name
        type: strz
        size: length_of_entry_name
      - id: num_of_entries
        type: u4
      - id: entries
        type: u4
        repeat: expr
        repeat-expr: num_of_entries
      - id: length
        type: u4
      - id: number
        type: u4
      - id: data
        type: header_data
        size: length
        repeat: expr
        repeat-expr: number
        
  header_data:
    seq:
      - id: unk1
        type: u4
      - id: unk2
        type: u4
 
  chnk:
    seq:
      - id: magic
        contents: "CHNK"
      - id: number_of_entry
        type: u4
      - id: entries
        type: chnk_entry
        repeat: expr
        repeat-expr: number_of_entry
  
  chnk_entry:
    seq:
      - id: unknown
        type: u4
        doc: |
          Unknown value, seems like it's strictly increasing.
          For a cutscene dialogue, setting it to 0 makes the text disappear.
          I think it's related to dialogue apparition timing, but I can't be sure.
      - id: string_size
        type: u4
      - id: string
        type: strz
        size: string_size

