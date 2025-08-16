# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'DynamiXplore'
copyright = '2025, Daniel Dia'
author = 'Daniel Dia'

# Not in line with your package version -- I know, it's annoying!
release = '0.5.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',    # Pull documentation from docstrings
    'sphinx.ext.napoleon',   # Support for NumPy and Google style docstrings

    # (1) Was causing build to crash.
    """'sphinx_gallery.gen_gallery', # Generate a gallery of examples"""
]

templates_path = ['_templates']
exclude_patterns = []


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

# (2) You did not add this to your requirements.txt:
html_theme = 'furo'

html_static_path = ['_static']

# (1) This code was preventing sphinx from compiling docs for me:
# uncomment and follow the steps for your examples directory
"""import os
import sys
sys.path.insert(0, os.path.abspath('../../'))

# -- Options for Sphinx-Gallery --
sphinx_gallery_conf = {
    # path to your example scripts
    'examples_dirs': '../../examples',
    # path to where to save gallery generated output
    'gallery_dirs': 'auto_examples',
}"""
