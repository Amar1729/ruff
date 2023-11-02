"""
Should emit:
DAR102 - on line 7, 16, 26.
"""


def foo(a):
    """
    Args:
        a: First argument.
        b: Second argument.
    """
    pass


def bar(x):
    """
    Args:
        y: Second argument.
        _z: Third argument.
    """
    pass


class A:
    def method(self, a):
        """
        Args:
            a: First argument.
            b: Second argument.
        """
        pass
