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
    type: sheet_header
  - id: chunk
    type: chnk

enums:
  characters:
    0x1: takumi
    0x2: takemaru
    0x3: hiruko
    0x4: darumi
    0x5: eito
    0x6: tsubasa
    0x7: gaku
    0x8: ima
    0x9: kako
    0xA: shouma
    0x12: sirei
    0x63: takumi_combat
    0x65: murvrum
    0xCA: karua
    0xC9: takumis_mom
    0x12e: sirei_cutscene
    0x130: announcement
    0x131: thought
    0x132: pa_system
    0x134: lock
    0x136: door

types:
  sheet_header:
    seq:
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
      - id: align
        type: u4
        if: (length_of_entry_name + num_of_entries * 4) % 8 != 0
      - id: data
        type: header_data
        size: length
        repeat: expr
        repeat-expr: number

  header_data:
    seq:
      - id: message_id
        type: u4
      - id: character
        type: u4
        enum: characters
 
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
      - id: entry_id
        type: u4
      - id: string_size
        type: u4
      - id: string
        type: strz
        size: string_size
