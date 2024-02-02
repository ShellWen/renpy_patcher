import renpy.script as script

def new_load_file(self, *args, **kwargs):
    print(f"self: {self}, args: {args}, kwargs: {kwargs}")
    return self.__original_load_file(*args, **kwargs)

print("Patching renpy.script.Script.load_file...")
script.Script.__original_load_file = script.Script.load_file
script.Script.load_file = new_load_file
