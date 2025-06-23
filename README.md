idGrab: A header generator for ID-engine (Keen: Galaxy) games.
--------------------------------------------------------------

idGrab is a small utility for generating header and info files for games based
on id Software's 1991 'id Engine', used in games such as Commander Keen 4–6,
Bio Menace, and others.

These games store all of their graphics (and a few other miscellaneous bits of
data) as numbered "chunks" in a file called ``EGAGRAPH.???`` (or ``CGAGRAPH``
or ``VGAGRAPH``, depending on the graphics mode used). In order for the game
to understand which chunk numbers corresponded to which image (and which
_type_ of image), a C (and assembly) header was generated.

idGrab — much like the iGrab tool originally used by id — can convert a
'script' containing the names and types of chunks into headers in a number of
different formats, suitable for use with different games, utilities, and
projects:

- iGrab-compatible C and TASM headers, usable with the original games' code,
  and many decompilation projects.
- [Omnispeak](http://davidgow.net/keen/omnispeak) compatible ``GFXCHUNK.CKx``
  variable files.
- [TED5](https://moddingwiki.shikadi.net/wiki/TED5)- and Omnispeak-compatible
  ``GFXINFOE`` files.
- [modId](https://github.com/owenmpierce/modId)-compatible ``.def`` files.


Note that idGrab does not actually import or export the graphics, and so
does not create or read ``EGAHEAD`` or ``EGADICT`` files: you probably want
to use [modId](https://github.com/owenmpierce/modId) or
[uGrab](https://k1n9duk3.shikadi.net/ugrab.html) for that.

For more information on the EGAGRAPH format, check out the
[Modding Wiki](https://moddingwiki.shikadi.net/wiki/EGAGraph_Format)


## How Do I Use It?

idGrab is a command-line utility, currently available for MS Windows and Linux.

It accepts the filename of an idGrab script (see below), followed by a series of actions or options:

``--gfxinfo [filename]``
 : Generates a `GFXINFO(E)` file for use with TED or Omnispeak
 
 ``--modid [filename]``
 : Writes a modid/ugrab compatible `.def` file.
 
 ``--omnispeak [filename]``
 : Generates an Omnispeak-compatible GFXCHUNK variable file.
 
 ``--igrab-header [filename]``
 : Creates a ``GRAPHEXT.H``/``GFXE_EXT.H`` C header file, in the same format as IGRAB.
 
 ``--igrab-asm``
 : Creates a ``GRAPHEXT.EQU``/``GFXE_EXT.EQU`` assembly header file, in the same format as IGRAB.
 
 ``--igrab-version [0.24 | 0.4]``
 : Emulate the output from a specific IGRAB version. IGRAB version 0.24 and 0.4 are supported, and have minor differences in how the headers are formatted. IGRAB 0.24 uses a separate ``#define`` for each chunk, IGRAB 0.4 puts them all in an ``enum``.
 
 ``--igrab-underscore-separator``
 : Add an underscore before chunk name suffixes. So, for example, a sprite called ``DEMOSIGN`` would be called ``DEMOSIGN_SPR`` instead of ``DEMOSIGNSPR``.
 
 Multiple options / commands can be combined in one invocation, and are read in the order they are specified.
 
 For example, to output both a GFXINFOE and Omnispeak GFXCHUNK file, you could use:
 
 ```
 ./idgrab scripts/keen4.idgrab --gfxinfo GFXINFOE.CK4 --omnispeak GFXCHUNK.CK4
 ```
 
 Or, to generate both C and assembly headers, in IGRAB 0.24 format:
 ```
 ./idgrab scripts/keen5.idgrab --igrab-version 0.24 --igrab-header GRAPHCK5.H --igrab-asm GRAPHCK5.EQU
 ```
 
 (Note that the ``--igrab-version`` option must come _before_ the ``--igrab-header`` and ``--igrab-asm`` options in order to take effect.)
 
## The idGrab Script Format

idGrab scripts are essentially an ordered list of chunk names or properties, grouped together by type (e.g., Fonts, Bitmaps, etc.) and/or lump (a named, contiguous group of chunks which are cached together).

You can also think of an idGrab script as a series of 'statements', each of which either add one or more chunks to the file, or alter their properties. Statements are followed by arguments, either strings (pieces of text, like chunk names), which are enclosed in double quotes (``"``), numbers (which are integers, in decimal), or lists, which are enclosed in braces (``{}``).

The following statements exist in an idGrab script.

``Extension``
: Gives the file extension to be used, as a string. Note that this is not used by idGrab for the actual output filenames (which are specified on the command line, see above), but may be included in comments or metadata. For example,
```
Extension "CK5"
```

``Fonts``
: Contains the list of fonts, as chunk names Traditionally, these have names ending in ``FONT``.  If there are any fonts, this section must come before any other chunks. For example,
```
Fonts {
	"MAINFONT"
	"WATCHFONT"
	"STARWARSFONT"
}
```

``Bitmaps``
: Contains the list of bitmaps (also known as pictures), as chunk names. The name is automatically decorated with ``PIC`` when generating an Omnispeak or IGRAB file. Bitmaps are often contained in Lumps, which can be nested within the ``Bitmap`` statement (see below). For example,
```
Bitmaps {
	"STANDALONE"
	Lump "MENU" {
		"MENU1"
		"MENU2"
	}
}
```

``BitmapsMasked``
: Contains the list of masked bitmaps (also known as masked pictures), as chunk names. These names are automatically decorated with ``PICM`` when generating Omnispeak or IGRAB files. As with ``Bitmap``, these can be included in Lumps, though this is not done in any of the existing games. For example:
```
BitmapsMasked {
	"WRISTWATCHSCREEN"
	"STATUSLEFT"
	"STATUSRIGHT"
}
```

``Sprites``
: Contains the list of sprites, as chunk names. These names are automatically decorated with ``SPR`` when generating an Omnispeak or IGRAB file. As with ``Bitmap``, these can be included in Lumps (see below). For example:
```
Sprites {
	Lump "PADDLE" {
		"PADDLE"
		"BALL0"
		"BALL1"
		"BALL2"
		"BALL3"
	}
	"DEMOSIGN"
	Lump "KEEN" {
		"KEENSTANDR"
		…
	}
}
```

``Lump``
: Groups a contiguous set of chunks into a named 'lump', allowing them to be cached as a unit. The ``Lump`` statement takes two arguments, a string for the lump name, and the list of chunks within. This is interleaved in the lists of Sprites or Bitmaps.

``Tiles8``
: Specifies the number of unmasked 8×8 pixel tiles. These are not individually named, and are stored as a single chunk. The number of tiles and number of this chunk will be output. For example:
```
Tiles8 104
```

``Tiles8Masked``
: Specifies the number of _masked_ 8×8 pixel tiles. As with unmasked 8×8 tiles, these are stored in a single chunk and are not individually named. For example:
```
Tiles8Masked 20
```

``Tiles16``
: Specifies the number of _unmasked_ (background) 16×16 pixel tiles. These are each stored in their own chunk, but are not individually named. These are usually used for backgrounds, and may be _sparse_ (i.e. some tiles may have no data): these tiles are included in the count. idGrab will output the number of chunk and the offset of the _first_ chunk. For example,
```
Tiles16 1200
```

``Tiles16Masked``
: Specifies the number of _masked_ (foreground/icon) 16×16 pixel tiles. Like their unmasked equivalents, these are each stored in their own chunk, but are not named. The first few of these are used as 'icons' by the TED5 editor, the rest as 'foreground' tiles. As above, they may be _sparse_, and the count given includes the 0-length sparse chunks. idGrab will output the number of chunks and the offset of the _first_ chunk. For example,
```
Files16Masked 2000
```

``B8000Text``
: Specifies a chunk in containing an 80×25 character VGA text-mode screen. See [ModdingWiki's format description](https://moddingwiki.shikadi.net/wiki/B800_Text). A name for the chunk should be provided, which will be used undecorated. For example,
```
B8000Text "ORDERSCREEN"
```

``Terminator``
: Specifies a chunk consisting of the Keen 4–6 'Terminator' intro's RLE-encoded images. ModdingWiki has a [brief description](https://moddingwiki.shikadi.net/wiki/EGAGraph_Format#Keen_4-6_Intro_Bitmaps) of their format. A name for the chunk should be provided, which will be used undecorated. For example,
```
Terminator "COMMANDER"
Terminator "KEEN"
```

``Article``
: Specifies a chunk containing an in-game 'article'. These are used for formatted text screens, including in-game help and story text. A name for the chunk should be provided, which will be used undecorated. However, note that the in-game help code often depends on the order of its article chunks, not their names. For example,
```
Article "STORY"
```

``Demo``
: Specifies a chunk containing a recorded demo. These chunks are numbered, but not explicitly named. (An automatic name of ``DEMOn`` will be provided.) The demo number must be specified as an argument. For example,
```
Demo 0
Demo 1
Demo 2
```

Example idGrab scripts (for Commander Keen 4–6) can be found in the ``scripts/`` directory.
