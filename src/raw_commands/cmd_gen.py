import os
import re
import sys

REGS = {
    'A',
    'B',
    'C',
    'D',
    'E',
    'F',
    'H',
    'L',
    'AF',
    'BC',
    'DE',
    'HL',
    '*',
    'n',
    'nn',
    'SP',
    'PC',
    'cc',
    'b',
    'r',
    '#',
    'r1',
    'r2',
    'Cc',
    'NZ',
    'Z',
    'NC',
}

def check_parens(string):
    if string and string[0] == '(':
        idx = string.find(')')
        if idx > 0:
            return string[:idx]
    return None

def check_second(string, params):
    if string in REGS:
            params.append(string)
    else: 
        paren = check_parens(string)
        if paren:
            params.append(paren)

def extract_front_cmd(line):
    line = line.lstrip()
    next = line.split(' ')[0]
    remain = line[len(next):]
    return next, remain

def extract_back_cmd(line):
    line = line.rstrip()
    back = line.split(' ')[-1]
    remain = line[:-len(back)]
    return back, remain

def hard_extract_n_r(line, n):
    if n == 1:
        if line in REGS:
            return [[line]]
        else:
            None
    else:
        possibles = []
        for reg in REGS:
            if line[:len(reg)] == reg:
                remain = line[len(reg):]
                possible = hard_extract_n_r(remain, n - 1)
                if possible is not None:
                    updated = [[reg] + pos for pos in possible]
                    possibles.extend(updated)
        if len(possibles) == 0:
            return None
        else:
            return possibles


def hard_extract_n(line, n):
    ret = hard_extract_n_r(line, n)
    if ret is None:
        raise Exception("can't find args")
    else:
        if len(ret) > 1:
            print("found multiple possible args.")
        return ret[0]

def extract_n(line, n):
    if n == 1:
        found, remain = extract_front_cmd(line)
        return [found], remain
    else:
        line = line.lstrip()
        idx = line.find('y')
        if idx == -1:
            idx = line.find(',')
            if idx == -1:
                hard_line = line.split(' ')[0]
                remain = line[len(hard_line):]
                return hard_extract_n(hard_line, n), remain
        found = line[:idx].strip()
        remain = line[idx + 1:]
        nexts, final_remain = extract_n(remain, n - 1)
        return [found] + nexts, final_remain

def extract_all(line):
    founds = []
    while line.strip():
        found, line = extract_front_cmd(line)
        founds.append(found)
    return founds 

def process_cmd_line(line, num_args, flag):
    cmd, line = extract_front_cmd(line)
    if flag:
        cmd += '_FLAG'
    if num_args > 0:
        args, line = extract_n(line, num_args)
    else:
        args, pot_line = extract_front_cmd(line)
        if args == '-/-':
            line = pot_line
        args = []
    args = [process_arg(arg) for arg in args]
    cycles, line = extract_back_cmd(line)
    op_code = extract_all(line)
    op_code = [process_opcode(op) for op in op_code]
    args_str = ','.join(args)
    op_code_str = ','.join(op_code)
    return '|'.join([cmd, args_str, op_code_str, cycles])

def line_to_params(line, cmd):
    removed = line.replace(cmd, '', 1).lstrip()
    second_removed = removed.replace(',', '').strip()
    params = []
    if removed[:2] in REGS:
        params.append(removed[:2])
        second_removed = removed[2:].replace(',', '').strip()
        check_second(second_removed, params)
    elif removed[:1] in REGS:
        params.append(removed[:1])
        second_removed = removed[1:].replace(',', '').strip()
        check_second(second_removed, params)
    else:
        paren = check_parens(removed)
        if paren:
            params.append(paren)
            end_idx = removed.find(')')
            second_removed = removed[end_idx + 1:].replace(',', '').strip()
            check_second(second_removed, params)
    return params

def process_opcode(opcode):
    return opcode.replace('T', '7')\
                 .replace('O', '0')\
                 .replace('Q', '2')\
                 .replace('S', '5')\
                 .replace('l', '1')\
                 .replace('I', '1')

def process_arg(arg):
    arg = arg.strip()\
             .replace('O', '0')\
             .replace('#', 'n')\
             .replace('*', 'n')\
             .replace('Cc', 'C')\
             .replace('c', 'C')
    return arg

def get_upper(word):
    chars = [c for c in word if c.isupper()]
    return ''.join(chars)

def get_reg_split(line):
    idx = line.find('y')
    if idx == -1:
        idx = line.find(',')
        if idx == -1:
            return line.rstrip(), ''
    
    return line[:idx], line[idx+1:]

def get_reg_name_prior(line):
    line = line.rstrip()
    default_ret = ('', line.strip())
    if not line:
        return default_ret
    elif line[-2:] in REGS:
        return line[-2:], line[:-2]
    elif line[-1] in REGS:
        if line[-1] == "*":
            return "n", line[:-1]
        else:
            return line[-1], line[:-1]
    elif line[-1] == ')':
        idx = line.find('(')
        if idx == -1:
            return default_ret
        return line[idx:], line[:idx]
    else:
        return default_ret

def get_cmd(line, subsec):
    subline = line[len(subsec):].lstrip()
    first_reg, _ = get_reg_split(subline)
    _, before_reg = get_reg_name_prior(first_reg)
    ret1 = get_upper(before_reg)
    ret2 = get_upper(subline.split(' ')[0])
    return [
        (ret1, len(line_to_params(subline, ret1))),
        (ret2, len(line_to_params(subline, ret2))),
    ]

def is_cmd(line, prev, cmds):
    key = line.split(' ')[0]
    return key in prev or key in cmds

def get_num_params(line, prev, cmds):
    key = line.split(' ')[0]
    if key in prev:
        if not prev[key][1]:
            cmds[key] = prev[key][0]
        return prev[key]
    else:
        return cmds[key], False

def num_to_subsection(num):
    return str(num) + "."

def num_to_section(num):
    return "3.3." + str(num) + "."

def line_not_found(line, secs):
    return all([sec not in line for sec in secs])

def gen_list(raw, write_file):
    file = open(raw, 'r')
    out_file = open(write_file, 'w')
    lines = file.readlines()
    subsection = 1
    section = 1
    find_cmds = False
    found_secs = set()
    cmds = {}
    prev_cmds = {}
    for line in lines:
        section_name = num_to_section(section)
        sub_name = num_to_subsection(subsection)
        if len(section_name) <= len(line) and \
            section_name == line[:len(section_name)]:
            out_file.write(section_name + '\n')
            section += 1
            subsection = 1
            found_secs.add(section_name)
            find_cmds = False
        elif len(sub_name) <= len(line) and \
            sub_name == line[:len(sub_name)] and \
            line_not_found(line, found_secs):
            out_file.write(sub_name + '\n')
            upds = filter(lambda x: len(x[0]) > 0, get_cmd(line, sub_name))
            prev_cmds = {}
            for upd in upds:
                if upd[0] in cmds and cmds[upd[0]] < upd[1]:
                    prev_cmds[upd[0]] = (upd[1], True)
                else:
                    prev_cmds[upd[0]] = (upd[1], False)
            subsection += 1
            find_cmds = False
        elif "Opcodes:" in line:
            find_cmds = True
        elif find_cmds and is_cmd(line, prev_cmds, cmds):
            num_params, flag = get_num_params(line, prev_cmds, cmds)
            cmd_line = process_cmd_line(line, num_params, flag)
            out_file.write(cmd_line + '\n')
    file.close()
    out_file.close()

def process_lines(path):
    file = open(path, 'r')
    lines = file.readlines()
    section = None
    subsection = None
    section_re = re.compile('[0-9]+\.[0-9]+\.[0-9]+\.')
    sub_re = re.compile('[0-9]+\.')
    opcodes = {}
    for line in lines:
        line = line.strip()
        if section_re.match(line) is not None:
            section = line
        elif sub_re.match(line) is not None:
            subsection = line
        else:
            cmds = line.split('|')
            cmd, args, opcode, cycles = cmds
            try:
                int(cycles)
            except ValueError:
                continue
            args = args.split(',')
            opcode = opcode.split(',')
            if len(opcode) > 1:
                if opcode[0] == 'CB':
                    opcode = '1' + opcode[1]
                else:
                    opcode = opcode[0]
            else:
                opcode = opcode[0]
            op_map = opcodes
            # while len(opcode) > 1:
            #     op = int(opcode[0], 16)
            #     if op not in op_map:
            #         op_map[op] = {}
            #     op_map = op_map[op]
            #     opcode = opcode[1:]
            op = int(opcode, 16)
            op_map[op] = [section, subsection, cmd, args, cycles]
    return opcodes

def inc_reg(reg):
    if reg[0] == '(':
        return '($FF00+' + reg[1:]
    else:
        return '$FF00' + reg

def inc_tup(tup):
    tup[3][1] = inc_reg(tup[3][1])

CHANGES = {
    ("3.3.1.", "5."): inc_tup,
}

def process_dict(file, opcodes, orig=True, prev_key=0):
    gen_more = {}
    for key, val in opcodes.items():
        if isinstance(val, dict):
            file.write('        if !sub_map.contains_key(&' + str(key) + ') {\n')
            file.write(f'            sub_map.insert({str(key)}, HashMap::new());\n')
            file.write('        }\n')
            process_dict(file, val, orig=False, prev_key=key)
        else:
            sec, sub, _, _, _ = val
            if (sec, sub) in CHANGES:
                CHANGES[(sec, sub)](val)
            _, _, cmd, regs, cycles = val
            if 'b' in regs:
                if cmd not in gen_more:
                    gen_more[cmd] = []
                gen_more[cmd].append((cmd, regs, cycles, key))
                continue
            fn_str = get_fn_str(cmd, regs, cycles)
            if orig:
                #file.write(f'        op_map.insert((None, {str(key)}), {fn_str});\n')
                file.write(f'        op_map.insert({str(key)}, {fn_str});\n')
            else:
                file.write(f'        sub_map.get(&{str(prev_key)}).unwrap().insert({str(key)}, {fn_str});\n')
                sub_str = get_sub_fn_str(prev_key, key)
                file.write(f'        op_map.insert((Some({str(prev_key)}), {str(key)}), *{sub_str});\n')

    for tups in gen_more.values():
        to_add = get_generated(tups)
        for (cmd, regs, cycles, key) in to_add:
            fn_str = get_fn_str(cmd, regs, cycles)
            if orig:
                file.write(f'        op_map.insert({str(key)}, {fn_str});\n')
    
def get_generated(tup_list):
    ret = []
    for (cmd, regs, cycles, key) in tup_list:
        for i in range(8):
            new_regs = [reg.replace('b', 'b' + str(i)) for reg in regs]
            new_key = key + i * len(tup_list)
            ret.append((cmd, new_regs, cycles, new_key))
    return ret


FLAGS = {
    'Z',
    'C',
    'H',
    'N',
}

def conv_to_rs_inp(reg, flag):
    if flag and reg[-1] in FLAGS:
        if len(reg) > 1 and reg[0] == 'N':
            prefix = 'RegExt::NFlag(Flag::'
        else:
            prefix = 'RegExt::Flag(Flag::'
        return prefix + reg[-1] + ')'
    if reg[0] == 'b':
        return 'RegExt::B(' + reg[1] + ')'
    if len(reg) > 1 and reg[-1] == 'H':
        num = int(reg[:-1], 16)
        return 'RegExt::H(' + str(num) + ')'
    if reg.isupper():
        return 'RegExt::Reg(Reg::' + reg + ')'
    else:
        return 'RegExt::' + reg.upper()


def process_regs(regs, flag):
    reg_inps = []
    mem_inps = []
    add_inps = []
    idxs = 0
    for raw_reg in regs:
        if not raw_reg:
            continue
        if raw_reg[0] == '(':
            idx = raw_reg.find(')')
            raw_reg = raw_reg[1:idx]
            mem_inps.append('true')
        else:
            mem_inps.append('false')
        plus_idx = raw_reg.find('+')

        if plus_idx == -1:
            add_inps.append('0')
        else:
            add_part = raw_reg[:plus_idx]
            raw_reg = raw_reg[plus_idx+1:]
            if add_part[0] == '$':
                add = int(add_part[1:], 16)
            else:
                add = int(add_part)
            add_inps.append(str(add))
        reg_inps.append(conv_to_rs_inp(raw_reg, flag))
        idxs += 1
    return [(reg_inps[i], mem_inps[i], add_inps[i]) for i in range(idxs)]


def create_cmd_inp(inp_tup):
    reg, mem, add = inp_tup
    return f"CmdInp::new({reg}, {mem}, {add})"    

def get_fn_str(cmd, regs, cycles):
    flag = "_flag" in cmd.lower()
    inps = process_regs(regs, flag)
    params = ', '.join([create_cmd_inp(inp) for inp in inps])
    if len(inps) > 0:
        params = ', ' + params
    return '|mother| {' + cmd.lower() + '(mother' + params + '); return ' + cycles + ';}'


def get_sub_fn_str(prev_key, key):
    return 'sub_map.get(&' + str(prev_key) + ').unwrap().get(&' + str(key) + ').unwrap()'


def write_body(file, opcodes):
    file.write('// Note: This file is generated. Check raw_commands/cmd_gen.py\n\n')
    file.write('use crate::cmd::*;\n')
    file.write('use crate::cpu::{Flag, Reg};\n')
    file.write('use crate::motherboard::Motherboard;\n\n')
    file.write('use std::collections::HashMap;\n\n')
    file.write('type OpInp = (Option<u8>, u8);\n')
    file.write('type CmdFns = fn(&mut Motherboard) -> u8;\n\n')
    file.write('pub struct OpCmds {\n')
    file.write('    op_map: HashMap<u16, CmdFns>,\n')
    #file.write('    sub_map: HashMap<u8, HashMap<u8, CmdFns>>,\n')
    file.write('}\n\n')
    file.write('impl OpCmds {\n')
    file.write('    pub fn new() -> Self {\n')
    file.write('        let mut op_map: HashMap<u16, CmdFns> = HashMap::new();\n')
    #file.write('        let mut sub_map: HashMap<u8, HashMap<u8, CmdFns>> = HashMap::new();\n')
    process_dict(file, opcodes)
    file.write('        Self {\n')
    file.write('            op_map,\n')
    #file.write('            sub_map,\n')
    file.write('        }\n')
    file.write('    }\n\n')
    file.write('    pub fn exe_op(&self, mother: &mut Motherboard, op: u16) -> u8 {\n')
    file.write('        self.op_map.get(&op).unwrap()(mother)\n')
    file.write('    }\n')
    file.write('}\n')

def write_file(path, opcodes):
    file = open(path, 'w')
    write_body(file, opcodes)
    file.close()

if __name__ == '__main__':
    MANUAL = "opcodes_manual.txt"
    CMDS_FILE = "opcodes.txt"
    RS_FILE = "op_cmds.rs"
    cur_path = os.path.dirname(sys.argv[0])
    inp_path = os.path.join(cur_path, MANUAL)
    out_path = os.path.join(cur_path, CMDS_FILE)
    rs_path = os.path.join(cur_path, '..', RS_FILE)
    gen_list(inp_path, out_path)
    op_dict = process_lines(out_path)
    write_file(rs_path, op_dict)
