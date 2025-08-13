# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'DynamiXplore'
copyright = '2025, Daniel Dia'
author = 'Daniel Dia'
release = '0.4.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',    # Pull documentation from docstrings
    'sphinx.ext.napoleon',   # Support for NumPy and Google style docstrings
    'sphinx_gallery.gen_gallery', # Generate a gallery of examples
]

templates_path = ['_templates']
exclude_patterns = []


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'furo'
html_static_path = ['_static']

import os
import sys
sys.path.insert(0, os.path.abspath('../../'))

# -- Options for Sphinx-Gallery --
sphinx_gallery_conf = {
    # path to your example scripts
    'examples_dirs': '../../examples',
    # path to where to save gallery generated output
    'gallery_dirs': 'auto_examples',
}
