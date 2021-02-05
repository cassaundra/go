#lang racket

(define chunk-size (make-parameter 5))

(define (parse-winner result)
  (match result
    [(regexp #rx"W\\+.+") 'white]
    [(regexp #rx"B\\+.+") 'black]
    [_ #f]))

(define (read-games sgf-directory)
  (match-let ([(list in out pid err info)
               (process (format "sgfinfo -m -rpropRE -q -nf -r ~a" sgf-directory))])
    (filter
     (lambda (game)
       ;; filter for valid games
       (and game
            (and (apply (lambda (a b) (and a b)) game))))
     (for/list ([line (in-lines in)])
       (match (string-split line)
         [(list moves result)
          (list (string->number moves) (parse-winner result))]
         [_ #f])
       ))
    ))

(define (winners-by-length games)
  (define table (make-hash))
  (for ([game games])
    (match-let ([(list moves winner) game])
      (let* ([chunk (* (chunk-size) (quotient moves (chunk-size)))]
             ;; [chunk-value]
             )
        (hash-update! table chunk
                      (λ (value)
                        (let ([black-winners (car value)]
                              [white-winners (cdr value)])
                          (case winner
                            [(black) (cons (add1 black-winners) white-winners)]
                            [(white) (cons black-winners (add1 white-winners))])))
                      (cons 0 0))
        )))
  table)

(define (median-moves games)
  (define moves
    (list->vector
     (sort (for/list ([game games])
             (car game))
           <)))
  (define count (vector-length moves))
  (if (zero? count)
      #f
      (/ (+ (vector-ref moves (quotient (sub1 count) 2))
            (vector-ref moves (quotient count 2)))
         2)))

;; print in a LaTeX/TikZ format

(module+ main
  (define sgf-directory
    (command-line
     #:program "analyze-pro-games"
     #:once-each
     [("-c" "--chunk-size") cs
                            "Size of chunks"
                            (chunk-size (string->number cs))]
     #:args (sgf-directory)
     sgf-directory))

  (define games (read-games sgf-directory))

  (let ([chunk-table (winners-by-length games)])
    (for ([chunk+winners (sort (hash->list chunk-table)
                               (λ (a b)
                                 (< (car a) (car b))))])
      (define winners (cdr chunk+winners))
      (let ([black-winners (car winners)]
            [white-winners (cdr winners)])
        (when (> (+ black-winners white-winners) 0)
          (printf "(~a,~a) "
                  (car chunk+winners)
                  (~r (* 100 (/ black-winners (+ black-winners white-winners)))
                      #:precision 4)))
        ))

        )
  (println))
