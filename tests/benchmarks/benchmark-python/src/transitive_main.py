import lib_b
from lib_b import TransitiveClass

def do_transitive() -> TransitiveClass:
    tc = lib_b.TransitiveClass()
    return tc

def do_transitive_specific() -> TransitiveClass:
    tc2 = TransitiveClass()
    return tc2
