//! Parses the subset of the Multiboot2 boot information structure we need:
//! the framebuffer tag GRUB fills in from VBE (BIOS) or GOP (UEFI). Same
//! kernel code path either way -- Multiboot2 abstracts over which firmware
//! actually set the mode up.

const TAG_TYPE_FRAMEBUFFER: u32 = 8;
const FRAMEBUFFER_TYPE_RGB: u8 = 1;

pub struct FramebufferInfo {
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub red_pos: u8,
    pub red_size: u8,
    pub green_pos: u8,
    pub green_size: u8,
    pub blue_pos: u8,
    pub blue_size: u8,
}

/// `info_ptr` is the pointer GRUB leaves in `ebx` at kernel entry, forwarded
/// here unchanged from `boot.asm`. Walks the tag list looking for the
/// framebuffer tag (type 8); only the direct-RGB color type is supported,
/// since that's what VBE/GOP linear framebuffers report.
pub unsafe fn find_framebuffer(info_ptr: *const u8) -> Option<FramebufferInfo> {
    let total_size = (info_ptr as *const u32).read_unaligned() as usize;
    let mut offset: usize = 8; // skip total_size + reserved

    while offset + 8 <= total_size {
        let tag = info_ptr.add(offset);
        let tag_type = (tag as *const u32).read_unaligned();
        let tag_size = (tag.add(4) as *const u32).read_unaligned() as usize;

        if tag_type == 0 {
            break; // terminator tag
        }

        if tag_type == TAG_TYPE_FRAMEBUFFER {
            let addr = (tag.add(8) as *const u64).read_unaligned();
            let pitch = (tag.add(16) as *const u32).read_unaligned();
            let width = (tag.add(20) as *const u32).read_unaligned();
            let height = (tag.add(24) as *const u32).read_unaligned();
            let bpp = *tag.add(28);
            let fb_type = *tag.add(29);

            if fb_type != FRAMEBUFFER_TYPE_RGB {
                return None;
            }

            return Some(FramebufferInfo {
                addr,
                pitch,
                width,
                height,
                bpp,
                red_pos: *tag.add(32),
                red_size: *tag.add(33),
                green_pos: *tag.add(34),
                green_size: *tag.add(35),
                blue_pos: *tag.add(36),
                blue_size: *tag.add(37),
            });
        }

        // Tags are 8-byte aligned; tag_size itself is not padded.
        offset += (tag_size + 7) & !7;
    }

    None
}
