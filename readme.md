# OpenMW BumpMap Translator

# What Is This?
OBMT is a tool to convert any meshes in your load order using Morrowind.exe style bumpmaps into openmw-compliant ones. It achieves this in three phases: 

1. Any textures whose names end with `_nm.dds` are renamed to end in `_n.dds`
2. References to textures ending with `_nm.dds` are removed from the mesh
3. Also checks a hardcoded list of `NiTextureEffect` nodes which are known to be invalid and removes those also.

This is a destructive process and the original assets are not backed up. However, the result is a qualitative visual improvement. 

See screenshots below. (later)

# Okay, How Do I Use It?

Navigate to the releases category on this page and simply download the build most relevant for your platform.

Double-click the binary once and it will chew through your entire load order and log whatever meshes it alters in a file in the same folder, called `translator.log`
