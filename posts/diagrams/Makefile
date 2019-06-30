TEXS := $(wildcard *.tex)

all: $(TEXS:.tex=.svg)

clean:
	@rm *.svg -f

%.svg: %.tex
	@latex $< > /dev/null
	@dvisvgm $(<:.tex=.dvi) > /dev/null
	-@sed "s/width='.*pt' //g" $@ > test.out; tee $@ < test.out > /dev/null
	-@rm $(<:.tex=.dvi) $(<:.tex=.aux) $(<:.tex=.log) test.out -f
