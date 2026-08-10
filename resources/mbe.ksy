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
    0x0: none
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
    0xB: nozomi
    0xC: kurara
    0xD: kyoshika
    0xE: yugamu
    0xF: moko
    0x10: eva
    0x11: shion
    0x12: sirei
    0x13: nigou
    0x63: takumi_combat_form
    0x65: murvrum
    0x66: pakron
    0x67: addamaque
    0x68: quenzelle
    0x69: parmith
    0x6A: eva_paragon
    0x6B: zen_ta
    0x6C: valla_garzo
    0x6D: szanshin
    0x6E: nyewgank
    0x6F: turamtammi
    0x70: dahl_xia
    0x71: vexhness
    0xC9: takumis_mom
    0xCA: karua_kashimiya
    0xCB: karua
    0xCC: mr_sirei
    0xCD: mr_nigou
    0xCE: person_in_leap_suit
    0xCF: ghostly_woman
    0xD0: figure_in_black
    0xD1: old_man
    0xD2: kamyuhn
    0xD3: villager
    0xD4: village_woman
    0xD5: village_boy
    0xD6: village_girl
    0xD7: kako_g
    0xD8: village_man
    0xD9: village_man_2
    0xDA: village_woman_2
    0xFB: other_takumi
    0xFC: other_takemaru
    0xFD: other_hiruko
    0xFE: other_darumi
    0xFF: other_eito
    0x100: other_tsubasa
    0x101: other_gaku
    0x102: other_ima
    0x103: other_kako
    0x104: other_shouma
    0x105: other_nozomi
    0x106: other_kurara
    0x107: other_kyoshika
    0x108: other_yugamu
    0x109: other_moko
    0x10A: other_eva
    0x10B: other_shion
    0x10C: other_sirei
    0x10D: other_nigou
    0x10E: other_vexhness
    0x10F: other_eito_2
    0x110: other_eito_3
    0x111: other_eito_4
    0x112: other_eito_5
    0x113: futuran_child
    0x114: sponsor
    0x115: sponsor_2
    0x116: sponsor_3
    0x117: sponsor_4
    0x118: sponsor_5
    0x119: sponsor_6
    0x11A: village_boy_2
    0x11B: futuran_girl
    0x11C: sponsor_kako_g
    0x11D: sponsor_eito
    0x12D: all
    0x12E: unknown
    0x12F: defense_system
    0x130: announcement
    0x131: thought
    0x132: pa_system
    0x133: bell
    0x134: lock
    0x135: doorbell
    0x136: door
    0x137: mother
    0x138: young_man
    0x139: father
    0x13A: humanity
    0x13B: takumi_ii
    0x13C: takumi_iii
    0x13D: takumi_iv
    0x13E: letter
    0x13F: shadowy_figure
    0x140: noise
    0x141: message
    0x142: doll
    0x143: takumi_v
    0x144: hiruko_v
    0x145: zombie_hiruko
    0x146: hiruko_ii
    0x147: all_four
    0x148: takumi_vi
    0x149: hiruko_vi
    0x14A: hirukos
    0x14C: shadow
    0x14D: gie_queen
    0x14E: fb
    0x14F: all_2
    0x150: nigou_question
    0x151: goldie
    0x152: nozomi_ii
    0x153: takumi_g
    0x154: gie
    0x155: takumi_vii
    0x156: hiruko_vii
    0x157: the_curse
    0x158: boy
    0x159: girl
    0x15A: kid
    0x15B: female_student
    0x15C: young_woman
    0x15D: middle_aged_man
    0x15E: middle_aged_woman
    0x15F: old_man_2
    0x160: old_woman
    0x161: eva_question
    0x162: moko_question
    0x163: vexhness_ii
    0x164: sponsor_unused
    0x165: evas_voice
    0x166: kakos_voice
    0x167: creature
    0x168: x
    0x169: retsnom
    0x16A: holy_jumonji_sword
    0x16B: darumarr
    0x16C: strange_invader
    0x16D: takumi_question
    0x16E: eito_aotsuki_2
    0x16F: eito_aotsuki_3
    0x170: eito_aotsuki_4
    0x171: eito_aotsuki_5
    0x172: eito_aotsuki_6
    0x173: interviewer
    0x174: announcement_2
    0x175: takumi_question_2
    0x176: hiruko_question
    0x177: sponsor_2_unused
    0x178: sponsor_3_unused
    0x179: sponsor_4_unused
    0x17A: sponsor_5_unused
    0x17B: sponsor_6_unused
    0x17D: gotoh
    0x17E: gies_voice
    0x17F: scream
    0x180: strange_voice
    0x181: vexhness_and_vexhness_ii
    0x182: sponsor_kako_g_unused
    0x183: sponsor_eito_unused
    0x184: kako_and_ima
    0x185: ima_and_kako
    0x186: takemaru_and_shouma
    0x187: tsubasa_kyoshika_yugamu
    0x188: kurara_and_nozomi
    0x189: nigou_and_eva
    0x18A: futuran_1
    0x18B: futuran_2
    0x18C: futuran_3
    0x18D: moko_and_kurara
    0x18E: takumi_and_hiruko

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
