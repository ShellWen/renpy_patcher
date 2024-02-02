import renpy.loader as loader

__original_load = loader.load


def new_load(*args, **kwargs):
    print(f"args: {args}, kwargs: {kwargs}")
    return __original_load(*args, **kwargs)

print("Patching renpy.loader.load...")
loader.load = new_load
